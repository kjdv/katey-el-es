extern crate certutils;
extern crate clap;
extern crate futures;
extern crate io_copy;
extern crate string_error;
extern crate tokio;
extern crate tokio_rustls;

use futures::future::{try_select, Either};
use io_copy::copy;
use std::sync::Arc;
use tokio::io::{split, stdin, stdout};
use tokio::net::TcpStream;
use tokio_rustls::rustls;
use tokio_rustls::TlsConnector;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let args = clap::App::new("katey-client")
        .author("Klaas de Vries")
        .about("tls-enabled telnet-like client")
        .arg(
            clap::Arg::with_name("address")
                .help("address to connect to, i.e. localhost:1729")
                .index(1)
                .required(true),
        )
        .arg(
            clap::Arg::with_name("root")
                .help("path to the file containing the root certificates, in .pem format.")
                .short("r")
                .long("root")
                .required(true)
                .takes_value(true)
        )
        .arg(
            clap::Arg::with_name("cert")
                .help("path to the file containing the certificate used for client authentication, in .pem format")
                .short("c")
                .long("cert")
                .requires("key")
                .takes_value(true)
        )
        .arg(
            clap::Arg::with_name("key")
                .help("path to the file containing the private key used for client authentication, in.pem format")
                .short("k")
                .long("key")
                .requires("cert")
                .takes_value(true)
        )
        .get_matches();

    let address = args.value_of("address").unwrap();

    let config = certutils::make_client_config(
        args.value_of("root").unwrap(),
        args.value_of("cert"),
        args.value_of("key"),
    )?;

    let mut runtime = tokio::runtime::Builder::new()
        .enable_all()
        .basic_scheduler()
        .build()?;

    let ret = runtime.block_on(async { handle(&address, config).await });

    // kludge: tokio's stdin is implemented using a background thread, and needs explicit shutdown
    runtime.shutdown_timeout(std::time::Duration::from_secs_f64(0.1));

    ret
}

async fn handle(address: &str, config: rustls::ClientConfig) -> Result<()> {
    let dom = domain(address);
    let connector = TlsConnector::from(Arc::new(config));
    let stream = TcpStream::connect(address).await?;
    let stream = connector.connect(certutils::dns_name(dom), stream).await?;

    let (rx, tx) = split(stream);
    let input = stdin();
    let output = stdout();

    let to = tokio::spawn(copy(input, tx));
    let from = tokio::spawn(copy(rx, output));

    match try_select(to, from).await {
        Ok(Either::Left((Ok(_), _))) => {
            eprintln!("local->remote closed");
            Ok(())
        }
        Ok(Either::Left((Err(to), _))) => {
            eprintln!("local->remote erred: {:?}", to);
            Err(to.into())
        }
        Ok(Either::Right((Ok(_), _))) => {
            eprintln!("remote->local closed");
            Ok(())
        }
        Ok(Either::Right((Err(from), _))) => {
            eprintln!("remote->local closed: {:?}", from);
            Err(from.into())
        }
        Err(Either::Left((e, _))) => {
            eprintln!("local->remote error: {:?}", e);
            Err(e.into())
        }
        Err(Either::Right((e, _))) => {
            eprintln!("remote->local error: {:?}", e);
            Err(e.into())
        }
    }
}

fn domain(address: &str) -> &str {
    match address.find(':') {
        Some(n) => &address[..n],
        None => address,
    }
}
