extern crate clap;
extern crate io_copy;
extern crate simple_logger;
extern crate tokio;

use io_copy::proxy;
use tokio::io::{split, stdin, stdout};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let args = clap::App::new("server")
        .author("Klaas de Vries")
        .about("simple telnet-like tcp client")
        .arg(
            clap::Arg::with_name("debug")
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
        .get_matches();

    if args.is_present("debug") {
        simple_logger::init()?;
    }

    let address = args.value_of("address").unwrap();

    let config = tcp_client::Config::new(address)
        .with_shutdown_timeout(std::time::Duration::from_secs_f64(0.1));
    let mut client = tcp_client::Client::new(config)?;

    client.run(handle)
}

async fn handle(stream: tcp_client::Stream) -> Result<()> {
    let stream = split(stream);
    let input = stdin();
    let output = stdout();

    proxy((input, output), stream).await
}
