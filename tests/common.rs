extern crate assert_cmd;
extern crate rustls;
extern crate webpki;

use rustls::{internal::pemfile, ClientSession, ServerSession};
use std::io::{Read, Write};
use std::sync::Arc;

pub fn certgen(args: &[&str]) -> assert_cmd::Command {
    let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("cargo bin");
    cmd.args(args);
    cmd.current_dir(current_dir());
    cmd
}

pub fn assert_valid_key(filename: &str) {
    read_key(&filename);
}

pub fn assert_valid_cert(filename: &str) {
    let certs = read_certs(&filename);
    assert_eq!(1, certs.len(), "expected 1 cert");
}

pub fn make_pair(key: &str, cert: &str, root: &str, name: &str) -> (ClientSession, ServerSession) {
    let server_cfg = Arc::new(make_server_config(&key, &cert));
    let client_cfg = Arc::new(make_client_config(&root));

    let client = rustls::ClientSession::new(&client_cfg, dns_name(name));
    let server = rustls::ServerSession::new(&server_cfg);

    (client, server)
}

// represent a connected session, inspired and simplified from rustls test sets
pub struct OtherSession<'a> {
    pub sess: &'a mut dyn rustls::Session,
}

impl<'a> Read for OtherSession<'a> {
    fn read(&mut self, mut b: &mut [u8]) -> std::io::Result<usize> {
        self.sess.write_tls(b.by_ref())
    }
}

impl<'a> Write for OtherSession<'a> {
    fn write(&mut self, mut b: &[u8]) -> std::io::Result<usize> {
        let r = self.sess.read_tls(b.by_ref())?;
        self.sess.process_new_packets().map_err(|e| {
            let e = format!("{}", e);
            std::io::Error::new(std::io::ErrorKind::Other, e)
        })?;
        Ok(r)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn current_dir() -> std::path::PathBuf {
    let path = std::env::current_exe().unwrap();
    let path = path.parent().unwrap();
    path.to_path_buf()
}

fn load_file(filename: &str) -> String {
    let mut path = current_dir();
    path.push(filename);
    let mut file = std::fs::File::open(path).expect("file exists");
    let mut content = String::new();
    file.read_to_string(&mut content).expect("file is readable");
    content
}

fn read_key(filename: &str) -> rustls::PrivateKey {
    let pem = load_file(filename);
    let mut reader = std::io::BufReader::new(pem.as_bytes());
    let keys = pemfile::pkcs8_private_keys(&mut reader).expect("cant load keys");
    assert_eq!(1, keys.len(), "expected 1 key");
    keys[0].clone()
}

fn read_certs(filename: &str) -> Vec<rustls::Certificate> {
    let cert = load_file(filename);
    let mut reader = std::io::BufReader::new(cert.as_bytes());
    pemfile::certs(&mut reader).expect("cant load cert file")
}

fn make_server_config(key: &str, cert: &str) -> rustls::ServerConfig {
    let mut cfg = rustls::ServerConfig::new(rustls::NoClientAuth::new());
    cfg.set_single_cert(read_certs(cert), read_key(key))
        .expect("setting cert and key");
    cfg
}

fn make_client_config(filename: &str) -> rustls::ClientConfig {
    let root = load_file(filename);
    let mut cfg = rustls::ClientConfig::new();
    let mut reader = std::io::BufReader::new(root.as_bytes());
    cfg.root_store
        .add_pem_file(&mut reader)
        .expect("add pem file");
    cfg
}

fn dns_name(name: &str) -> webpki::DNSNameRef<'_> {
    webpki::DNSNameRef::try_from_ascii_str(name).unwrap()
}
