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
            SubCommand::with_name("sign")
                .about("sign a certificate request")
                .arg(ca_arg.clone().required(true).index(1))
                .arg(name_arg.clone().required(true).index(2)),
        )
        .subcommand(
            SubCommand::with_name("tree")
                .about("create a full tree of a root ca and multiple hosts")
                .arg(
                    Arg::with_name("ca")
                        .help("name for the root ca")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("hosts")
                        .help("name(s) for the hosts")
                        .required(true)
                        .multiple(true),
                ),
        )
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    let subcommands: &[(&str, fn(&ArgMatches))] = &[("root", root), ("sign", sign), ("tree", tree)];
    for (subcommand, handler) in subcommands {
        if let Some(sub_args) = args.subcommand_matches(subcommand) {
            handler(sub_args);
        }
    }
}

fn root(args: &ArgMatches) {
    do_root(name_arg(&args));
}

fn do_root(name: &str) {
    let names = vec!["localhost".to_string(), name.to_string()];

    let mut params = CertificateParams::new(names);
    params.is_ca = rcgen::IsCa::Ca(BasicConstraints::Unconstrained);
    let cert = Certificate::from_params(params).expect("certificate generation");

    let filename = key_filename(name);
    dump(filename.as_str(), &cert.serialize_private_key_pem());

    let filename = cert_filename(name);
    dump(filename.as_str(), &cert.serialize_pem().expect("ca pem"));
}

fn sign(args: &ArgMatches) {
    do_sign(ca_arg(&args), name_arg(&args));
}

fn do_sign(ca: &str, name: &str) {
    let key = key_filename(&ca);
    let ca = cert_filename(&ca);

    let key = load(key.as_str());
    let ca = load(ca.as_str());

    let key = KeyPair::from_pem(&key).expect("keypair loading");
    let ca = CertificateParams::from_ca_cert_pem(ca.as_str(), key).expect("cert loading");
    let ca = Certificate::from_params(ca).expect("certificate");

    let cert = generate(&name);

    let filename = key_filename(&name);
    dump(filename.as_str(), &cert.serialize_private_key_pem());

    let filename = cert_filename(&name);
    let cert = cert.serialize_pem_with_signer(&ca).expect("signing");
    dump(filename.as_str(), &cert);
}

fn tree(args: &ArgMatches) {
    let ca = args.value_of("ca").unwrap();
    do_root(ca);

    let hosts: Vec<&str> = args.values_of("hosts").unwrap().collect();
    for host in hosts.iter() {
        // to do, this wastes by reading the same ca in a loop, the same one we have just written
        do_sign(ca, host);
    }
}

fn generate(name: &str) -> Certificate {
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

fn key_filename(basename: &str) -> String {
    format!("{}-key.pem", basename)
}

fn name_arg<'a>(args: &'a ArgMatches) -> &'a str {
    args.value_of("name").expect("name arg to be provided")
}

fn ca_arg<'a>(args: &'a ArgMatches) -> &'a str {
    args.value_of("ca").expect("name arg to be provided")
}
