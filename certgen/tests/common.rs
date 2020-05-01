extern crate assert_cmd;
extern crate certutils;
extern crate rand;
extern crate rcgen;
extern crate rustls;
extern crate webpki;

use rand::Rng;
use rustls::{ClientSession, ServerSession};
use std::io::{Read, Write};
use std::sync::Arc;

pub fn certgen(args: &[&str]) -> assert_cmd::Command {
    let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("cargo bin");
    cmd.args(args);
    cmd.current_dir(current_dir());
    cmd
}

pub fn assert_valid_key(filename: &str) {
    certutils::read_key(&file_path(&filename)).expect("valid key");
}

pub fn assert_valid_cert(filename: &str) {
    let certs = certutils::read_certs(&file_path(&filename)).expect("valid cert");
    assert_eq!(1, certs.len(), "expected 1 cert");
}

pub fn make_server(key: &str, cert: &str, root: &str) -> ServerSession {
    let cfg = Arc::new(
        certutils::make_server_config(&file_path(&cert), &file_path(&key), Some(&file_path(&root)))
            .expect("server config"),
    );
    rustls::ServerSession::new(&cfg)
}

pub fn make_client(key: &str, cert: &str, root: &str, name: &str) -> ClientSession {
    let cfg = Arc::new(
        certutils::make_client_config(&file_path(&cert), &file_path(&key), &file_path(&root))
            .expect("client config"),
    );
    rustls::ClientSession::new(&cfg, dns_name(name))
}

pub fn unique_name(head: &str) -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwzyz012345679";

    let mut rng = rand::thread_rng();
    let tail: String = (0..10)
        .map(|_| {
            let idx = rng.gen_range(0, CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    format!("{}-{}", head, tail)
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

fn file_path(filename: &str) -> String {
    let mut path = current_dir();
    path.push(filename);
    path.to_str().unwrap().to_string()
}

fn dns_name(name: &str) -> webpki::DNSNameRef<'_> {
    webpki::DNSNameRef::try_from_ascii_str(name).unwrap()
}
