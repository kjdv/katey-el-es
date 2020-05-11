extern crate certutils;
extern crate futures;
extern crate log;
extern crate rustls;
extern crate string_error;
extern crate tokio;
extern crate tokio_rustls;

use futures::future::Future;
use std::marker::{Send, Sync};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal::unix::{signal, SignalKind};
use tokio_rustls::TlsAcceptor;

pub type Stream = tokio_rustls::server::TlsStream<tokio::net::TcpStream>;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone)]
pub struct Config {
    port: u16,
    threaded: bool,
    shutdown_timeout: std::time::Duration,
    tls: rustls::ServerConfig,
}

impl Config {
    pub fn new(port: u16) -> Config {
        Config {
            port,
            threaded: false,
            shutdown_timeout: std::time::Duration::from_secs(1),
            tls: rustls::ServerConfig::new(rustls::NoClientAuth::new()),
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

    pub fn with_certificate_and_key_files(
        &mut self,
        certfile: &str,
        keyfile: &str,
    ) -> Result<&mut Self> {
        let cert = certutils::read_certs(certfile)?;
        let key = certutils::read_key(keyfile)?;
        self.tls.set_single_cert(cert, key)?;
        Ok(self)
    }

    pub fn with_client_authentication(&mut self, root_certfile: &str) -> Result<&mut Self> {
        let mut store = rustls::RootCertStore { roots: vec![] };
        certutils::read_certs(root_certfile)?
            .iter()
            .try_for_each(|c| store.add(c))?;
        let auth = rustls::AllowAnyAuthenticatedClient::new(store);
        self.tls.set_client_certificate_verifier(auth);
        Ok(self)
    }
}

pub struct Server {
    config: Config,
    runtime: Option<tokio::runtime::Runtime>,
}

impl Server {
    pub fn new(config: Config) -> Result<Server> {
        log::info!("creating server");

        let mut runtime = tokio::runtime::Builder::new();

        if config.threaded {
            log::info!("using multi-threaded scheduler");
            runtime.threaded_scheduler();
        } else {
            log::info!("using single-threaded scheduler");
            runtime.basic_scheduler();
        }

        let runtime = runtime.enable_all().build()?;

        Ok(Server {
            config,
            runtime: Some(runtime),
        })
    }

    pub fn run<F, R>(&mut self, handler: F) -> Result<()>
    where
        F: Fn(Stream) -> R + Send + Sync + Copy + 'static,
        R: Future + Send,
    {
        match self.runtime.take() {
            Some(mut rt) => {
                let res = rt.block_on(async { self.serve_with_graceful_shutdown(handler).await });
                self.wait(rt);
                res.map_err(|e| e.into())
            }
            None => Err(string_error::static_err("can not run the server twice")),
        }
    }

    fn wait(&self, rt: tokio::runtime::Runtime) {
        log::debug!(
            "waiting for {:?} to shut down",
            self.config.shutdown_timeout
        );
        rt.shutdown_timeout(self.config.shutdown_timeout);
    }

    async fn serve_with_graceful_shutdown<F, R>(&self, handler: F) -> std::io::Result<()>
    where
        F: Fn(Stream) -> R + Send + Sync + Copy + 'static,
        R: Future + Send,
    {
        tokio::select! {
            x = self.serve(handler) => x,
            x = self.wait_for_signal(SignalKind::interrupt()) => x,
            x = self.wait_for_signal(SignalKind::terminate()) => x,
        }
    }

    async fn serve<F, R>(&self, handler: F) -> std::io::Result<()>
    where
        F: Fn(Stream) -> R + Send + Sync + Copy + 'static,
        R: Future + Send,
    {
        let listen_address = format!("0.0.0.0:{}", self.config.port);

        let cfg = self.config.tls.clone();
        let acceptor = TlsAcceptor::from(Arc::new(cfg));

        log::info!("listening on {:?}", listen_address);
        let mut listener = TcpListener::bind(listen_address).await?;

        loop {
            let (stream, remote_address) = listener.accept().await?;
            log::info!("accepted connection from {}", remote_address);

            let acceptor = acceptor.clone();

            tokio::spawn(async move {
                match acceptor.accept(stream).await {
                    Ok(stream) => {
                        handler(stream).await;
                        log::info!("closing connection from {}", remote_address);
                    }
                    Err(e) => {
                        log::warn!("not accepted: {}", e);
                    }
                }
            });
        }
    }

    async fn wait_for_signal(&self, kind: SignalKind) -> std::io::Result<()> {
        let mut sig = signal(kind)?;
        sig.recv().await;
        log::info!("received signal {:?}", kind);
        Ok(())
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        if let Some(rt) = self.runtime.take() {
            self.wait(rt);
        }
    }
}
