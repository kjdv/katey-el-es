extern crate clap;
extern crate io_copy;
extern crate simple_logger;
extern crate tokio;

use io_copy::proxy;
use tokio::io::{split, stdin, stdout};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let args = clap::App::new("katey-client")
        .author("Klaas de Vries")
        .about("tls-enabled telnet-like client")
        .arg(clap::Arg::with_name("debug")
            .help("enable debug logging")
            .short("d")
            .long("debug"),
        )
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

    if args.is_present("debug") {
        simple_logger::init()?;
    }

    let address = args.value_of("address").unwrap();

    let mut config = katey_client::Config::new(address);
    config
        .with_shutdown_timeout(std::time::Duration::from_secs_f64(0.1))
        .with_root(args.value_of("root").unwrap())
        .expect("config");

    if args.is_present("cert") {
        assert!(args.is_present("key"));

        config.with_certificate_and_key_files(
            args.value_of("cert").unwrap(),
            args.value_of("key").unwrap(),
        )?;
    }

    let mut client = katey_client::Client::new(config)?;
    client.run(handle).and_then(|r| r.map_err(|e| e.into()))
}

async fn handle(stream: katey_client::Stream) -> std::io::Result<()> {
    let stream = split(stream);
    let input = stdin();
    let output = stdout();

    proxy((input, output), stream).await
}
