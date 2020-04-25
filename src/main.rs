extern crate clap;
extern crate rcgen;

use std::io::Write;
use clap::{App, Arg, ArgMatches, AppSettings, SubCommand};
use rcgen::*;

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
            SubCommand::with_name("root").about("generates a root certificate and key")
            .arg(name_arg.clone())
        )
        .subcommand(SubCommand::with_name("request").about("generates a certificate request"))
        .subcommand(SubCommand::with_name("sign").about("sign a certificate request"))
        .subcommand(
            SubCommand::with_name("selfsigned").about("create a self-signed certificate")
            .arg(name_arg.clone())
        )
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    let subcommands: &[(&str, fn(&ArgMatches))] =
        &[("root", root), ("request", request), ("sign", sign), ("selfsigned", selfsigned)];
    for (subcommand, handler) in subcommands {
        if let Some(sub_args) = args.subcommand_matches(subcommand) {
            handler(sub_args);
        }
    }
}

fn root(args: &ArgMatches) {
    let name = args.value_of("name").unwrap();
    let names = vec!["localhost".to_string(), name.to_string()];

    let params = CertificateParams::new(names);
    let cert = Certificate::from_params(params).expect("certificate generation");

    let pem = cert.serialize_pem().expect("serialize to pem");
    let filename = ca_filename(&args);
    dump(filename.as_str(), &pem);

    let pem = cert.serialize_private_key_pem();
    let filename = key_filename(&args);
    dump(filename.as_str(), &pem);
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

fn key_filename(args: & ArgMatches) -> String {
    let basename = name_arg(args);
    format!("{}-key.pem", basename)
}

fn name_arg<'a>(args: &'a ArgMatches) -> &'a str {
    args.value_of("name").expect("name arg to be provided")
}
