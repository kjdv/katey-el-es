extern crate clap;
extern crate tokio;
extern crate futures;

use tokio::net::{TcpStream};
use tokio::io::{stdin, stdout, copy};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    let args = clap::App::new("server")
        .author("Klaas de Vries")
        .about("simple telnet-like tcp client")
        .arg(
            clap::Arg::with_name("host")
                .help("host to connect to")
                .index(1)
                .required(true),
        )
        .arg(
            clap::Arg::with_name("port")
                .help("port to connect to")
                .short("p")
                .long("port")
                .index(2)
                .required(true)
        )
        .get_matches();

    let address = format!("{}:{}", args.value_of("host").unwrap(), args.value_of("port").unwrap());

    handle(&address).await?;

    Ok(())
}

async fn handle(address: &str) -> Result<()> {
    let mut stream = TcpStream::connect(address).await?;
    let (mut rx, mut tx) = stream.split();
    let mut input = stdin();
    let mut output = stdout();

    let to = copy(&mut input, &mut tx);
    let from = copy(&mut rx, &mut output);

    futures::future::try_join(to, from).await?;
    Ok(())
}
