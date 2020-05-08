extern crate clap;
extern crate io_copy;
extern crate simple_logger;
extern crate tokio;

use io_copy::proxy;
use tokio::io::{split, stdin, stdout};
use tokio::net::TcpStream;

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

    let mut runtime = tokio::runtime::Builder::new()
        .enable_all()
        .basic_scheduler()
        .build()?;

    runtime.block_on(async { handle(&address).await })?;

    // kludge: tokio's stdin is implemented using a background thread, and needs explicit shutdown
    runtime.shutdown_timeout(std::time::Duration::from_secs_f64(0.1));

    Ok(())
}

async fn handle(address: &str) -> Result<()> {
    let stream = TcpStream::connect(address).await?;
    let stream = split(stream);
    let input = stdin();
    let output = stdout();

    proxy((input, output), stream).await
}
