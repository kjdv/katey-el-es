extern crate certutils;
extern crate clap;
extern crate futures;
extern crate log;
extern crate simple_logger;
extern crate tokio;
extern crate tokio_rustls;

use std::str::FromStr;
use tokio_rustls::rustls;
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;

use futures::future::try_select;
use std::sync::Arc;
use tokio::io::{copy, split, AsyncRead, AsyncWrite};
use tokio::net::{TcpListener, TcpStream};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    let args = clap::App::new("katey-el-es")
        .author("Klaas de Vries")
        .about("TLS proxy")
        .arg(
            clap::Arg::with_name("debug")
                .help("enable debug logging")
                .short("d")
                .long("debug"),
        )
        .arg(
            clap::Arg::with_name("listen")
                .help("port to listen on")
                .index(1)
                .required(true)
        )
        .arg(
            clap::Arg::with_name("forward")
                .help("address to forward to, i.e. localhost:1729")
                .index(2)
                .required(true)
        )
        .arg(
            clap::Arg::with_name("cert")
                .help("path to the file containing the certificate, in .pem format")
                .short("c")
                .long("cert")
                .takes_value(true)
                .required(true)
        )
        .arg(
            clap::Arg::with_name("key")
                .help("path to the file containing the private key, in.pem format")
                .short("k")
                .long("key")
                .takes_value(true)
                .required(true)
        )
        .arg(
            clap::Arg::with_name("root")
                .help("path to the file containing the root certificates, in .pem format.")
                .short("r")
                .long("root")
                .takes_value(true)
        )
        .arg(
            clap::Arg::with_name("client_auth")
                .help("enable client authentication, takes the path to the root certificate store, in .pem format")
                .short("a")
                .long("authenticate")
                .takes_value(true)
        )
        .get_matches();

    let level = if args.is_present("debug") {
        log::Level::Debug
    } else {
        log::Level::Info
    };

    simple_logger::init_with_level(level)?;

    log::debug!("arguments are config file is {:?}", args);

    let listen_address = format!("0.0.0.0:{}", args.value_of("listen").unwrap());
    let forward_address = args.value_of("forward").unwrap();
    log::info!(
        "setting up to listen at {} and forward to {}",
        listen_address,
        forward_address
    );

    let config = make_config(&args)?;

    serve(config, &listen_address, &forward_address).await?;

    Ok(())
}

async fn serve(
    config: rustls::ServerConfig,
    listen_address: &str,
    forward_address: &str,
) -> Result<()> {
    let forward_address = String::from_str(forward_address)?;
    let acceptor = TlsAcceptor::from(Arc::new(config));
    let mut listener = TcpListener::bind(listen_address).await?;
    log::info!("listening on {:?}", listen_address);

    loop {
        let (stream, remote_address) = listener.accept().await?;
        log::info!("accepted connection from {}", remote_address);

        let acceptor = acceptor.clone();
        let forward_address = forward_address.clone();

        tokio::spawn(async move {
            match acceptor.accept(stream).await {
                Ok(stream) => match TcpStream::connect(&forward_address).await {
                    Ok(forward) => {
                        handle(stream, forward).await;
                        log::info!("closing connection from {}", remote_address);
                    }
                    Err(e) => {
                        log::error!("could not forward: {}", e);
                    }
                },
                Err(e) => {
                    log::warn!("not accepted: {}", e);
                }
            }
        });
    }
}

async fn handle<IO>(from_stream: TlsStream<IO>, to_stream: TcpStream)
where
    IO: AsyncRead + AsyncWrite + std::marker::Unpin,
{
    let (mut from_rx, mut from_tx) = split(from_stream);
    let (mut to_rx, mut to_tx) = split(to_stream);

    let to = copy(&mut from_rx, &mut to_tx);
    let from = copy(&mut to_rx, &mut from_tx);

    match try_select(to, from).await {
        Ok(_) => {
            log::debug!("clean exit");
        }
        Err(_) => {
            log::error!("closing due to error");
        }
    };
}

fn make_config(args: &clap::ArgMatches) -> Result<rustls::ServerConfig> {
    let key = args.value_of("key").unwrap();
    let cert = args.value_of("cert").unwrap();

    log::info!("using certificate {} with private key {}", cert, key);

    let maybe_client_auth = args.value_of("client_auth");
    if let Some(root_path) = maybe_client_auth {
        log::info!(
            "enabling client authentication using root ca's in {}",
            root_path
        );
    }

    certutils::make_server_config(key, cert, maybe_client_auth)
}
