extern crate certutils;
extern crate clap;
extern crate tokio;
extern crate tokio_rustls;

use tokio_rustls::rustls;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    let args = clap::App::new("katey-client")
        .author("Klaas de Vries")
        .about("tls-enabled telnet-like client")
        .arg(
            clap::Arg::with_name("address")
                .help("address to connect to, i.e. localhost:1729")
                .index(1)
                .required(true),
        )
        .arg(
            clap::Arg::with_name("root")
                .help("path to the file containing the root certificates, in .pem format.")
                .short("r")
                .long("root")
                .required(true)
                .takes_value(true)
        )
        .arg(
            clap::Arg::with_name("cert")
                .help("path to the file containing the certificate used for client authentication, in .pem format")
                .short("c")
                .long("cert")
                .requires("key")
                .takes_value(true)
        )
        .arg(
            clap::Arg::with_name("key")
                .help("path to the file containing the private key used for client authentication, in.pem format")
                .short("k")
                .long("key")
                .requires("cert")
                .takes_value(true)
        )
        .get_matches();

    let address = args.value_of("address").unwrap();

    let _config = certutils::make_client_config(
        args.value_of("root").unwrap(),
        args.value_of("cert"),
        args.value_of("key"),
    )?;

    println!("{}", address);

    Ok(())
}
