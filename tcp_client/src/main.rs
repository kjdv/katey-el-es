extern crate clap;
extern crate futures;
extern crate string_error;
extern crate tokio;

use futures::future::try_select;
use tokio::io::{copy, stdin, stdout};
use tokio::net::TcpStream;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let args = clap::App::new("server")
        .author("Klaas de Vries")
        .about("simple telnet-like tcp client")
        .arg(
            clap::Arg::with_name("address")
                .help("address to connect to, i.e. localhost:1729")
                .index(1)
                .required(true),
        )
        .get_matches();

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
    let mut stream = TcpStream::connect(address).await?;
    let (mut rx, mut tx) = stream.split();
    let mut input = stdin();
    let mut output = stdout();

    let to = copy(&mut input, &mut tx);
    let from = copy(&mut rx, &mut output);

    match try_select(to, from).await {
        Ok(_) => Ok(()),
        Err(e) => Err(string_error::into_err(format!("{:?}", e))),
    }
}
