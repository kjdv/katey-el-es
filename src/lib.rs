extern crate rcgen;
use rcgen::*;

pub fn generate_root(names: Vec<String>) -> (String, String) {
    let params = CertificateParams::new(names);
    let cert = Certificate::from_params(params).expect("certificate generation");

    let key = cert.serialize_private_key_pem();
    let cert = cert.serialize_pem().expect("ca pem");
    (key, cert)
}


#[cfg(test)]
mod tests {
    extern crate rustls;
    extern crate webpki;

    use super::*;
    use std::sync::Arc;
    use std::io::{Read, Write};
    use rustls::{internal::pemfile, Session};

    fn read_key(pem: &str) -> rustls::PrivateKey {
        let mut reader = std::io::BufReader::new(pem.as_bytes());
        let keys = pemfile::pkcs8_private_keys(&mut reader).expect("cant load keys");
        assert_eq!(1, keys.len(), "expected 1 key");
        keys[0].clone()
    }

    fn read_certs(cert: &str) -> Vec<rustls::Certificate> {
        let mut reader = std::io::BufReader::new(cert.as_bytes());
        pemfile::certs(&mut reader).expect("cant load cert file")
    }

    fn make_server_config(key: &str, cert: &str) -> rustls::ServerConfig {
        let mut cfg = rustls::ServerConfig::new(rustls::NoClientAuth::new());
        cfg.set_single_cert(read_certs(cert), read_key(key))
            .expect("setting cert and key");
        cfg
    }

    fn make_client_config(root: &str) -> rustls::ClientConfig {
        let mut cfg = rustls::ClientConfig::new();
        let mut reader = std::io::BufReader::new(root.as_bytes());
        cfg.root_store.add_pem_file(&mut reader).expect("add pem file");
        cfg
    }

    // represent a connected session, inspired and simplified from rustls test sets
    struct OtherSession<'a> {
        sess: &'a mut dyn rustls::Session
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

    fn dns_name(name: &'static str) -> webpki::DNSNameRef<'_> {
        webpki::DNSNameRef::try_from_ascii_str(name).unwrap()
    }

    fn make_pair(key: &str, cert: &str, root: &str) -> (rustls::ClientSession, rustls::ServerSession) {
        let server_cfg = Arc::new(make_server_config(&key, &cert));
        let client_cfg = Arc::new(make_client_config(&root));

        let client = rustls::ClientSession::new(&client_cfg, dns_name("test"));
        let server = rustls::ServerSession::new(&server_cfg);

        (client, server)
    }

    #[test]
    fn generates_valid_key_and_ca() {
        let names = vec!["localhost".to_string(), "test".to_string()];
        let (key, cert) = generate_root(names);

        make_server_config(&key, &cert);
    }

    #[test]
    fn server_config_is_usable() {
        let names = vec!["localhost".to_string(), "test".to_string()];
        let (key, cert) = generate_root(names);

        let (mut client, mut server) = make_pair(&key, &cert, &cert);
        assert_eq!(true, client.is_handshaking());

        client.complete_io(&mut OtherSession{sess: &mut server}).unwrap();
        assert_eq!(false, client.is_handshaking());
    }

    #[test]
    fn client_rejects_bad_cert() {
        let names = vec!["localhost".to_string(), "test".to_string()];
        let (key, cert) = generate_root(names.clone());
        let (_, root) = generate_root(names);

        let (mut client, mut server) = make_pair(&key, &cert, &root);
        assert_eq!(true, client.is_handshaking());

        let mut other = OtherSession{sess: &mut server};
        client.complete_io(&mut other).expect_err("should reject");
    }
}
