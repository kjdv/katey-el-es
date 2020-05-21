extern crate certutils;
extern crate futures;
extern crate log;
extern crate rustls;
extern crate string_error;
extern crate tokio;

use futures::future::Future;
use std::io::Read;
use std::sync::Arc;
use tokio_rustls::TlsConnector;

pub type Stream = tokio_rustls::client::TlsStream<tokio::net::TcpStream>;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone)]
pub struct Config<'a> {
    address: &'a str,
    threaded: bool,
    shutdown_timeout: std::time::Duration,
    tls: rustls::ClientConfig,
}

impl Config<'_> {
    pub fn new(address: &str) -> Config {
        Config {
            address,
            threaded: false,
            shutdown_timeout: std::time::Duration::from_secs(1),
            tls: rustls::ClientConfig::new(),
        }
    }

    pub fn with_threading(&mut self, threaded: bool) -> &mut Self {
        self.threaded = threaded;
        self
    }

    pub fn with_shutdown_timeout(&mut self, timeout: std::time::Duration) -> &mut Self {
        self.shutdown_timeout = timeout;
        self
    }

    pub fn with_root(&mut self, root: &str) -> Result<&mut Self> {
        let root = Config::load_file(root)?;
        let mut reader = std::io::BufReader::new(root.as_bytes());
        self.tls
            .root_store
            .add_pem_file(&mut reader)
            .map_err(|_| string_error::static_err("failed to add certificate to root store"))?;
        Ok(self)
    }

    pub fn with_certificate_and_key_files(
        &mut self,
        certfile: &str,
        keyfile: &str,
    ) -> Result<&mut Self> {
        let cert = certutils::read_certs(certfile)?;
        let key = certutils::read_key(keyfile)?;
        self.tls.set_single_client_cert(cert, key)?;
        Ok(self)
    }

    fn load_file(filename: &str) -> Result<String> {
        let mut file = std::fs::File::open(filename)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(content)
    }
}

pub struct Client<'a> {
    config: Config<'a>,
    runtime: Option<tokio::runtime::Runtime>,
}

impl Client<'_> {
    pub fn new(config: Config) -> Result<Client> {
        log::info!("creating client");

        let mut runtime = tokio::runtime::Builder::new();

        if config.threaded {
            log::info!("using multi-threaded scheduler");
            runtime.threaded_scheduler();
        } else {
            log::info!("using single-threaded scheduler");
            runtime.basic_scheduler();
        }

        let runtime = runtime.enable_all().build()?;

        Ok(Client {
            config,
            runtime: Some(runtime),
        })
    }

    pub fn run<F, R>(&mut self, handler: F) -> Result<R::Output>
    where
        F: Fn(Stream) -> R,
        R: Future,
    {
        match self.runtime.take() {
            Some(mut rt) => {
                let res = rt.block_on(async {
                    let dom = Client::domain(self.config.address);
                    let cfg = self.config.tls.clone();
                    let connector = TlsConnector::from(Arc::new(cfg));
                    let stream = tokio::net::TcpStream::connect(self.config.address).await?;
                    let stream = connector.connect(certutils::dns_name(dom), stream).await?;
                    Ok(handler(stream).await)
                });
                self.wait(rt);
                res
            }
            None => Err(string_error::static_err("can not run the client twice")),
        }
    }

    fn wait(&self, rt: tokio::runtime::Runtime) {
        log::debug!(
            "waiting for {:?} to shut down",
            self.config.shutdown_timeout
        );
        rt.shutdown_timeout(self.config.shutdown_timeout);
    }

    fn domain(address: &str) -> &str {
        match address.find(':') {
            Some(n) => &address[..n],
            None => address,
        }
    }
}

impl Drop for Client<'_> {
    fn drop(&mut self) {
        if let Some(rt) = self.runtime.take() {
            self.wait(rt);
        }
    }
}
