extern crate clap;
extern crate rcgen;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use rcgen::*;
use std::io::Write;

fn main() {
    let name_arg = Arg::with_name("name")
        .help("name for this root")
        .short("n")
        .long("name")
        .default_value("sample");

    let args = App::new("certgen")
        .about("Generates TLS Certificates")
        .version("0.1.0")
        .subcommand(
            SubCommand::with_name("root")
                .about("generates a root certificate and key")
                .arg(name_arg.clone()),
        )
        .subcommand(SubCommand::with_name("request").about("generates a certificate request"))
        .subcommand(SubCommand::with_name("sign").about("sign a certificate request"))
        .subcommand(
            SubCommand::with_name("selfsigned")
                .about("create a self-signed certificate")
                .arg(name_arg.clone()),
        )
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    let subcommands: &[(&str, fn(&ArgMatches))] = &[
        ("root", root),
        ("request", request),
        ("sign", sign),
        ("selfsigned", selfsigned),
    ];
    for (subcommand, handler) in subcommands {
        if let Some(sub_args) = args.subcommand_matches(subcommand) {
            handler(sub_args);
        }
    }
}

fn root(args: &ArgMatches) {
    let name = args.value_of("name").unwrap();
    let names = vec!["localhost".to_string(), name.to_string()];

    let (key, cert) = generate_root(names);

    let filename = key_filename(&args);
    dump(filename.as_str(), &key);

    let filename = ca_filename(&args);
    dump(filename.as_str(), &cert);
}

fn generate_root(names: Vec<String>) -> (String, String) {
    let params = CertificateParams::new(names);
    let cert = Certificate::from_params(params).expect("certificate generation");

    let key = cert.serialize_private_key_pem();
    let cert = cert.serialize_pem().expect("ca pem");
    (key, cert)
}

fn request(_args: &ArgMatches) {
    println!("implement me!")
}

fn sign(_args: &ArgMatches) {
    println!("implement me!")
}

fn selfsigned(args: &ArgMatches) {
    let name = args.value_of("name").unwrap();
    let names = vec!["localhost".to_string(), name.to_string()];

    let cert = generate_simple_self_signed(names).expect("certificate generation");
    let pem = cert.serialize_pem().expect("serialize to pem");

    let filename = ca_filename(&args);
    dump(filename.as_str(), &pem);
}

fn dump(filename: &str, content: &str) {
    let mut file = std::fs::File::create(filename).expect("file write");
    file.write_all(content.as_bytes()).expect("write");
}

fn ca_filename(args: &ArgMatches) -> String {
    let basename = name_arg(args);
    format!("{}-ca.pem", basename)
}

fn key_filename(args: &ArgMatches) -> String {
    let basename = name_arg(args);
    format!("{}-key.pem", basename)
}

fn name_arg<'a>(args: &'a ArgMatches) -> &'a str {
    args.value_of("name").expect("name arg to be provided")
}

#[cfg(test)]
mod tests {
    extern crate rustls;

    use super::*;
    use rustls::internal::pemfile;

    fn read_key(pem: String) -> rustls::PrivateKey {
        let mut reader = std::io::BufReader::new(pem.as_bytes());
        let keys = pemfile::pkcs8_private_keys(&mut reader).expect("cant load keys");
        assert_eq!(1, keys.len(), "expected 1 key");
        keys[0].clone()
    }

    fn read_certs(cert: String) -> Vec<rustls::Certificate> {
        let mut reader = std::io::BufReader::new(cert.as_bytes());
        pemfile::certs(&mut reader).expect("cant load cert file")
    }

    fn make_config(key: String, cert: String) -> rustls::ServerConfig {
        let mut cfg = rustls::ServerConfig::new(rustls::NoClientAuth::new());
        cfg.set_single_cert(read_certs(cert), read_key(key)).expect("setting cert and key");
        cfg
    }

    #[test]
    fn generates_valid_key_and_ca() {
        let names = vec!["localhost".to_string(), "test".to_string()];
        let (key, cert) = generate_root(names);

        make_config(key, cert);
    }
}
