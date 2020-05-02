extern crate rustls;

use std::io::Read;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn make_server_config(
    certfile: &str,
    keyfile: &str,
    client_auth: Option<&str>,
) -> Result<rustls::ServerConfig> {
    let auth = if let Some(root_path) = client_auth {
        let mut store = rustls::RootCertStore { roots: vec![] };

        let certs = read_certs(root_path)?;
        for c in certs.iter() {
            store.add(c)?;
        }
        rustls::AllowAnyAuthenticatedClient::new(store)
    } else {
        rustls::NoClientAuth::new()
    };

    let mut cfg = rustls::ServerConfig::new(auth);
    let cert = read_certs(certfile)?;
    let key = read_key(keyfile)?;
    cfg.set_single_cert(cert, key)?;
    Ok(cfg)
}

pub fn make_client_config(
    root: &str,
    cert: Option<&str>,
    key: Option<&str>,
) -> Result<rustls::ClientConfig> {
    let root = load_file(root)?;

    let mut cfg = rustls::ClientConfig::new();
    let mut reader = std::io::BufReader::new(root.as_bytes());
    cfg.root_store
        .add_pem_file(&mut reader)
        .map_err(|_| string_error::new_err("failed to add certificate to root store"))?;

    if let Some(cert) = cert {
        let cert = read_certs(cert)?;
        let key = read_key(key.expect("certificate needs a key"))?;
        cfg.set_single_client_cert(cert, key)?;
    }

    Ok(cfg)
}

pub fn read_key(filename: &str) -> Result<rustls::PrivateKey> {
    let pem = load_file(filename)?;
    let mut reader = std::io::BufReader::new(pem.as_bytes());
    let keys = rustls::internal::pemfile::pkcs8_private_keys(&mut reader)
        .map_err(|_| string_error::new_err("failed to load key"))?;

    match keys.len() {
        1 => Ok(keys[0].clone()),
        _ => Err(string_error::new_err("expected a single key in keyfile")),
    }
}

pub fn read_certs(filename: &str) -> Result<Vec<rustls::Certificate>> {
    let cert = load_file(filename)?;
    let mut reader = std::io::BufReader::new(cert.as_bytes());
    rustls::internal::pemfile::certs(&mut reader)
        .map_err(|_| string_error::new_err("failed to load certificates"))
}

pub fn dns_name(name: &str) -> webpki::DNSNameRef<'_> {
    webpki::DNSNameRef::try_from_ascii_str(name).unwrap()
}

fn load_file(filename: &str) -> Result<String> {
    let mut file = std::fs::File::open(filename)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}
