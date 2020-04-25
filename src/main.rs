extern crate clap;

fn main() {
    let args = clap::App::new("certgen")
        .about("Generates TLS Certificates")
        .version("0.1.0")
        .subcommand(
            clap::SubCommand::with_name("root").about("generates a root certificate and key"),
        )
        .subcommand(clap::SubCommand::with_name("request").about("generates a certificate request"))
        .subcommand(clap::SubCommand::with_name("sign").about("sign a certificate request"))
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    let subcommands: &[(&str, fn(&clap::ArgMatches))] =
        &[("root", root), ("request", request), ("sign", sign)];
    for (subcommand, handler) in subcommands {
        if let Some(sub_args) = args.subcommand_matches(subcommand) {
            handler(sub_args);
        }
    }
}

fn root(args: &clap::ArgMatches) {
    println!("implement me!")
}

fn request(args: &clap::ArgMatches) {
    println!("implement me!")
}

fn sign(args: &clap::ArgMatches) {
    println!("implement me!")
}
