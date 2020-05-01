extern crate clap;
extern crate log;
extern crate simple_logger;
extern crate tokio;
extern crate rustls;
extern crate string_error;

use std::io::Read;
use tokio::net::{TcpListener, TcpStream};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    let args = clap::App::new("katey-el-es")
        .author("Klaas de Vries")
        .about("TLS proxy")
        .arg(
            clap::Arg::with_name("debug")
                .help("enable debug logging")
                .short("d")
                .long("debug"),
        )
        .arg(
            clap::Arg::with_name("listen")
                .help("port to listen on")
                .index(1)
                .required(true)
        )
        .arg(
            clap::Arg::with_name("forward")
                .help("address to forward to, i.e. localhost:1729")
                .index(2)
                .required(true)
        )
        .arg(
            clap::Arg::with_name("cert")
                .help("path to the file containing the certificate, in .pem format")
                .short("c")
                .long("cert")
                .takes_value(true)
                .required(true)
        )
        .arg(
            clap::Arg::with_name("key")
                .help("path to the file containing the private key, in.pem format")
                .short("k")
                .long("key")
                .takes_value(true)
                .required(true)
        )
        .arg(
            clap::Arg::with_name("root")
                .help("path to the file containing the root certificates, in .pem format.")
                .short("r")
                .long("root")
                .takes_value(true)
        )
        .arg(
            clap::Arg::with_name("client_auth")
                .help("enable client authentication, takes the path to the root certificate store, in .pem format")
                .short("a")
                .long("authenticate")
                .takes_value(true)
        )
        .get_matches();

    let level = if args.is_present("debug") {
        log::Level::Debug
    } else {
        log::Level::Info
    };

    simple_logger::init_with_level(level)?;

    log::debug!("arguments are config file is {:?}", args);

    let listen_address = format!(
        "0.0.0.0:{}",
        args.value_of("listen").unwrap()
    );
    let forward_address = args.value_of("forward").unwrap();
    log::info!("setting up to listen at {} and forward to {}", listen_address, forward_address);

    let config = make_config(&args)?;

    Ok(())
}

fn make_config(args: &clap::ArgMatches) -> Result<rustls::ServerConfig> {
    let key = args.value_of("key").unwrap();
    let cert = args.value_of("cert").unwrap();

    log::info!("using certificate {} with private key {}", cert, key);

    let auth = if let Some(root_path) = args.value_of("client_auth") {
        log::info!("enabling client authentication using root ca's in {}", root_path);

        let mut store = rustls::RootCertStore { roots: vec![] };

        let certs = read_certs(root_path)?;
        for c in certs.iter() {
            store.add(c)?;
        }
        rustls::AllowAnyAuthenticatedClient::new(store)
    } else {
        log::info!("not enabling client authentication");
        rustls::NoClientAuth::new()
    };

    let mut cfg = rustls::ServerConfig::new(auth);
    let cert = read_certs(cert)?;
    let key = read_key(key)?;
    cfg.set_single_cert(cert, key)?;
    Ok(cfg)
}

fn read_key(filename: &str) -> Result<rustls::PrivateKey> {
    let pem = load_file(filename)?;
    let mut reader = std::io::BufReader::new(pem.as_bytes());
    let keys = rustls::internal::pemfile::pkcs8_private_keys(&mut reader)
        .map_err(|_| string_error::new_err("failed to load key"))?;

    match keys.len() {
        1 => Ok(keys[0].clone()),
        _ => Err(string_error::new_err("expected a single key"))
    }
}

fn read_certs(filename: &str) -> Result<Vec<rustls::Certificate>> {
    let cert = load_file(filename)?;
    let mut reader = std::io::BufReader::new(cert.as_bytes());
    rustls::internal::pemfile::certs(&mut reader)
        .map_err(|_| string_error::new_err("failed to load certs"))
}

fn load_file(filename: &str) -> Result<String> {
    let mut file = std::fs::File::open(filename)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}