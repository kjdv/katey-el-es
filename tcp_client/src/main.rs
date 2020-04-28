extern crate clap;

use std::io::{Read, Write};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
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

    handle(&address)?;

    Ok(())
}

fn handle(address: &str) -> Result<()> {
    let mut stream = std::net::TcpStream::connect(address)?;
    let mut buf = [0; 4096];

    loop {
        let line = prompt()?;
        if line.is_empty() {
            break;
        }

        stream.write_all(line.as_bytes())?;
        stream.flush()?;
        let n = stream.read(&mut buf)?;

        if n == 0 {
            break;
        }

        std::io::stdout().write_all(&buf[0..n])?;
        std::io::stdout().flush()?;
    }

    Ok(())
}

fn prompt() -> Result<String> {
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    Ok(line)
}
