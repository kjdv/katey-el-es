extern crate clap;
extern crate log;
extern crate simple_logger;
extern crate tcp_server;
extern crate tokio;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let args = clap::App::new("echo server")
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

    let port = args.value_of("port").unwrap().parse()?;
    let config = tcp_server::Config::new(port);
    let mut server = tcp_server::Server::new(config)?;

    server.run(handle)
}

async fn handle(mut stream: tcp_server::TcpStream) {
    let (mut rx, mut tx) = stream.split();
    if let Err(e) = tokio::io::copy(&mut rx, &mut tx).await {
        log::error!("error: {}", e);
    }
}
