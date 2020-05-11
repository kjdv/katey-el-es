extern crate clap;
extern crate io_copy;
extern crate log;
extern crate simple_logger;
extern crate tls_server;

use io_copy::proxy;
use std::str::FromStr;
use std::sync::Arc;
use tokio::io::split;
use tokio::net::TcpStream;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
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
            clap::Arg::with_name("threads")
                .help("enable multi-threaded server")
                .long("threads"),
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

    let listen_port: u16 = args.value_of("listen").unwrap().parse().expect("a number");
    let forward_address = args.value_of("forward").unwrap();
    log::info!(
        "setting up to listen at {} and forward to {}",
        listen_port,
        forward_address
    );

    let mut config = tls_server::Config::new(listen_port);
    config.with_threading(args.is_present("threads"));
    config.with_certificate_and_key_files(
        args.value_of("cert").unwrap(),
        args.value_of("key").unwrap(),
    )?;
    if let Some(root) = args.value_of("client_auth") {
        config.with_client_authentication(root)?;
    }

    let mut server = tls_server::Server::new(config)?;

    let forward_address = String::from_str(forward_address)?;
    server.run(move |stream| async move {
        forward_address.clone();
        match TcpStream::connect(forward_address).await {
            Ok(forward) => {
                handle(stream, forward);
            }
            Err(e) => {
                log::error!("could not forward: {}", e);
            }
        };
    })
}

async fn handle(from_stream: tls_server::Stream, to_stream: TcpStream) {
    let from_stream = split(from_stream);
    let to_stream = split(to_stream);

    let _ = proxy(from_stream, to_stream).await;
}
