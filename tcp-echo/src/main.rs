extern crate clap;
extern crate io_copy;
extern crate log;
extern crate simple_logger;
extern crate tcp_server;

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
            clap::Arg::with_name("public")
                .help("open publicly, not just localhost")
                .long("public"),
        )
        .arg(
            clap::Arg::with_name("threads")
                .help("enable multi-threaded server")
                .long("threads"),
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
    let config = tcp_server::Config::new(port)
        .with_public(args.is_present("public"))
        .with_threading(args.is_present("threads"));
    let mut server = tcp_server::Server::new(config)?;

    server.run(handle)
}

async fn handle(mut stream: tcp_server::TcpStream) -> Result<()> {
    let (rx, tx) = stream.split();
    io_copy::copy(rx, tx).await.map_err(|e| e.into())
}
