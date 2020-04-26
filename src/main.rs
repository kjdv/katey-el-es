extern crate clap;
extern crate rcgen;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use rcgen::*;
use std::io::{Read, Write};

fn main() {
    let name_arg = Arg::with_name("name").help("name for this certificate");
    let ca_arg = Arg::with_name("ca").help("name for the certificate authority");

    let args = App::new("certgen")
        .about("Generates TLS Certificates")
        .version("0.1.0")
        .subcommand(
            SubCommand::with_name("root")
                .about("generates a root certificate and key")
                .arg(name_arg.clone().required(true).index(1)),
        )
        .subcommand(
            SubCommand::with_name("request")
                .about("generates a certificate request")
                .arg(name_arg.clone().required(true).index(1)),
        )
        .subcommand(
            SubCommand::with_name("sign")
                .about("sign a certificate request")
                .arg(name_arg.clone().required(true).index(1))
                .arg(ca_arg.clone().required(true).index(2)),
        )
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    let subcommands: &[(&str, fn(&ArgMatches))] =
        &[("root", root), ("request", request), ("sign", sign)];
    for (subcommand, handler) in subcommands {
        if let Some(sub_args) = args.subcommand_matches(subcommand) {
            handler(sub_args);
        }
    }
}

fn root(args: &ArgMatches) {
    let name = args.value_of("name").unwrap();
    let names = vec!["localhost".to_string(), name.to_string()];

    let mut params = CertificateParams::new(names);
    params.is_ca = rcgen::IsCa::Ca(BasicConstraints::Unconstrained);
    let cert = Certificate::from_params(params).expect("certificate generation");

    let filename = key_filename(name_arg(&args));
    dump(filename.as_str(), &cert.serialize_private_key_pem());

    let filename = cert_filename(name_arg(&args));
    dump(filename.as_str(), &cert.serialize_pem().expect("ca pem"));
}

fn request(args: &ArgMatches) {
    let cert = generate(&args);

    let filename = key_filename(name_arg(&args));
    dump(filename.as_str(), &cert.serialize_private_key_pem());

    let filename = request_filename(name_arg(&args));
    dump(
        filename.as_str(),
        &cert.serialize_request_pem().expect("ca pem"),
    );
}

fn sign(args: &ArgMatches) {
    let key = key_filename(ca_arg(&args));
    let ca = cert_filename(ca_arg(&args));

    let key = load(key.as_str());
    let ca = load(ca.as_str());

    let key = KeyPair::from_pem(&key).expect("keypair loading");
    let ca = CertificateParams::from_ca_cert_pem(ca.as_str(), key).expect("cert loading");
    let ca = Certificate::from_params(ca).expect("certificate");

    let cert = generate(&args);

    let filename = key_filename(name_arg(&args));
    dump(filename.as_str(), &cert.serialize_private_key_pem());

    let filename = cert_filename(name_arg(&args));
    let cert = cert.serialize_pem_with_signer(&ca).expect("signing");
    dump(filename.as_str(), &cert);
}

fn generate(args: &ArgMatches) -> Certificate {
    let name = args.value_of("name").unwrap();
    let names = vec!["localhost".to_string(), name.to_string()];

    let params = CertificateParams::new(names);
    Certificate::from_params(params).expect("certificate generation")
}

fn dump(filename: &str, content: &str) {
    let mut file = std::fs::File::create(filename).expect("file write");
    file.write_all(content.as_bytes()).expect("write");
}

fn load(filename: &str) -> String {
    let mut file = std::fs::File::open(filename).expect("file open");
    let mut content = String::new();
    file.read_to_string(&mut content).expect("file read");
    content
}

fn cert_filename(basename: &str) -> String {
    format!("{}-cert.pem", basename)
}

fn request_filename(basename: &str) -> String {
    format!("{}-request.pem", basename)
}

fn key_filename(basename: &str) -> String {
    format!("{}-key.pem", basename)
}

fn name_arg<'a>(args: &'a ArgMatches) -> &'a str {
    args.value_of("name").expect("name arg to be provided")
}

fn ca_arg<'a>(args: &'a ArgMatches) -> &'a str {
    args.value_of("ca").expect("name arg to be provided")
}
