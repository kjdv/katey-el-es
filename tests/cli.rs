extern crate assert_cmd;
extern crate rustls;

use rustls::internal::pemfile;
use std::io::Read;

fn current_dir() -> std::path::PathBuf {
    let path = std::env::current_exe().unwrap();
    let path = path.parent().unwrap();
    path.to_path_buf()
}

fn certgen(args: &[&str]) -> assert_cmd::Command {
    let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("cargo bin");
    cmd.args(args);
    cmd.current_dir(current_dir());
    cmd
}

fn load_file(filename: &str) -> String {
    let mut path = current_dir();
    path.push(filename);
    let mut file = std::fs::File::open(path).expect("file exists");
    let mut content = String::new();
    file.read_to_string(&mut content).expect("file is readable");
    content
}

fn assert_valid_key(filename: &str) {
    let pem = load_file(filename);
    let mut reader = std::io::BufReader::new(pem.as_bytes());
    let keys = pemfile::pkcs8_private_keys(&mut reader).expect("cant load keys");
    assert_eq!(1, keys.len(), "expected 1 key");
}

fn assert_valid_cert(filename: &str) {
    let pem = load_file(filename);
    let mut reader = std::io::BufReader::new(pem.as_bytes());
    let certs = pemfile::certs(&mut reader).expect("cant load cert file");
    assert_eq!(1, certs.len(), "expected 1 cert");
}

#[test]
fn subcommand_is_mandatory() {
    let mut cmd = certgen(&[]);
    cmd.unwrap_err();
}

#[test]
fn root_needs_name_arg() {
    let mut cmd = certgen(&["root"]);
    cmd.unwrap_err();

    let mut cmd = certgen(&["root", "_"]);
    cmd.unwrap();
}

#[test]
fn root_generates_key_and_ca() {
    certgen(&["root", "sample-root"]).ok().unwrap();

    assert_valid_key("sample-root-key.pem");
    assert_valid_cert("sample-root-ca.pem");
}
