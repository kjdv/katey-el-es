extern crate clap;
extern crate log;
extern crate simple_logger;
extern crate tokio;

use tokio::net::{TcpListener, TcpStream};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    let args = clap::App::new("server")
        .author("Klaas de Vries")
        .about("simple tcp echo server")
        .arg(
            clap::Arg::with_name("debug")
                .help("enable debug logging")
                .short("d")
                .long("debug"),
        )
        .arg(
            clap::Arg::with_name("listen")
                .help("interface to listen on, i.e. 127.0.0.1 or 0.0.0.0")
                .short("l")
                .long("listen")
                .default_value("127.0.0.1"),
        )
        .arg(
            clap::Arg::with_name("port")
                .help("port to bind to")
                .short("p")
                .long("port")
                .default_value("1729"),
        )
        .get_matches();

    let level = if args.is_present("debug") {
        log::Level::Debug
    } else {
        log::Level::Info
    };

    simple_logger::init_with_level(level)?;

    log::debug!("arguments are config file is {:?}", args);

    let address = format!(
        "{}:{}",
        args.value_of("listen").unwrap(),
        args.value_of("port").unwrap()
    );
    serve(&address).await?;

    Ok(())
}

async fn serve(address: &str) -> Result<()> {
    let mut listener = TcpListener::bind(address).await?;
    log::info!("listening on {:?}", address);

    loop {
        let (stream, _) = listener.accept().await?;

        tokio::spawn(async move {
            handle(stream).await;
        });
    }
}

async fn handle(mut stream: TcpStream) {
    let (mut rx, mut tx) = stream.split();
    match tokio::io::copy(&mut rx, &mut tx).await {
        Ok(_) => log::info!("done, closing handler"),
        Err(e) => log::error!("error: {}", e)
    }
}
