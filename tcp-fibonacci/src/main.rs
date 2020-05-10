extern crate clap;
extern crate log;
extern crate simple_logger;
extern crate tcp_server;
extern crate tokio;

use std::time::Duration;
use tcp_server::TcpStream;
use tokio::io::AsyncWriteExt;
use tokio::time::delay_for;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let args = clap::App::new("fibonacci server")
        .author("Klaas de Vries")
        .about("simple demo server, prints N Fibonacci numbers and closes the connection")
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
                .default_value("1730"),
        )
        .arg(
            clap::Arg::with_name("n")
                .help("the number of Fibonacci numbers to produce")
                .short("n")
                .long("number")
                .default_value("10")
                .validator(validate_n),
        )
        .arg(
            clap::Arg::with_name("delay")
                .help("delay between the produced numbers")
                .short("i")
                .long("interval")
                .default_value("1")
                .validator(validate_delay),
        )
        .get_matches();

    let level = if args.is_present("debug") {
        log::Level::Debug
    } else {
        log::Level::Info
    };

    simple_logger::init_with_level(level)?;

    log::debug!("arguments are {:?}", args);

    let n: u32 = args.value_of("n").unwrap().parse().unwrap();
    let delay: f64 = args.value_of("delay").unwrap().parse().unwrap();
    let delay = Duration::from_secs_f64(delay);

    let port = args.value_of("port").unwrap().parse()?;
    let config = tcp_server::Config::new(port)
        .with_public(args.is_present("public"))
        .with_threading(args.is_present("threads"));
    let mut server = tcp_server::Server::new(config)?;

    server.run(move |stream| async move {
        if let Err(e) = handle(stream, n, delay).await {
            log::error!("error: {}", e);
        }
    })
}

async fn handle(mut stream: TcpStream, n: u32, delay: Duration) -> Result<()> {
    let mut a = 0;
    let mut b = 1;

    for _ in 0..n {
        let msg = format!("{}\n", a);
        stream.write_all(msg.as_bytes()).await?;
        stream.flush().await?;

        let wait = delay_for(delay);
        wait.await;

        b += a;
        a = b - a;
    }

    Ok(())
}

fn validate_n(v: String) -> std::result::Result<(), String> {
    v.parse::<u32>().map_err(|e| format!("{}", e))?;
    Ok(())
}

fn validate_delay(v: String) -> std::result::Result<(), String> {
    v.parse::<f64>().map_err(|e| format!("{}", e))?;
    Ok(())
}
