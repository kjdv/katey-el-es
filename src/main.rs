extern crate clap;
extern crate rcgen;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use rcgen::*;
use std::io::Write;

fn main() {
    let name_arg = Arg::with_name("name").help("name for this certificate");

    let args = App::new("certgen")
        .about("Generates TLS Certificates")
        .version("0.1.0")
        .subcommand(
            SubCommand::with_name("root")
                .about("generates a root certificate and key")
                .arg(name_arg.clone().required(true).index(1)),
        )
        .subcommand(SubCommand::with_name("request").about("generates a certificate request"))
        .subcommand(SubCommand::with_name("sign").about("sign a certificate request"))
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

    let params = CertificateParams::new(names);
    let cert = Certificate::from_params(params).expect("certificate generation");

    let filename = key_filename(&args);
    dump(filename.as_str(), &cert.serialize_private_key_pem());

    let filename = ca_filename(&args);
    dump(filename.as_str(), &cert.serialize_pem().expect("ca pem"));
}

fn request(_args: &ArgMatches) {
    println!("implement me!")
}

fn sign(_args: &ArgMatches) {
    println!("implement me!")
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
