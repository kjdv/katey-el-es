extern crate futures;
extern crate log;
extern crate string_error;
extern crate tokio;

use futures::future::Future;
use std::marker::{Send, Sync};
use tokio::net::TcpListener;

pub use tokio::net::TcpStream;
pub use tokio::runtime::Handle;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct Config {
    port: u16,
    public: bool,
    threaded: bool,
    shutdown_timeout: std::time::Duration,
}

impl Config {
    pub fn new(port: u16) -> Config {
        Config {
            port,
            public: false,
            threaded: false,
            shutdown_timeout: std::time::Duration::from_secs(1),
        }
    }

    pub fn with_public(&mut self) -> &mut Self {
        self.public = true;
        self
    }

    pub fn with_threads(&mut self) -> &mut Self {
        self.threaded = true;
        self
    }

    pub fn with_shutdown_timeout(&mut self, timeout: std::time::Duration) -> &mut Self {
        self.shutdown_timeout = timeout;
        self
    }
}

pub struct Server {
    config: Config,
    runtime: Option<tokio::runtime::Runtime>,
}

impl Server {
    pub fn new(config: Config) -> Result<Server> {
        log::debug!("creating tokio runtime");

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
        F: Fn(TcpStream) -> R + Sync + 'static,
        R: Future + Send + 'static,
        R::Output: Send + 'static,
    {
        match self.runtime.take() {
            Some(mut rt) => Ok(rt.block_on(async { self.serve(handler).await })?),
            None => Err(string_error::static_err("can not run the server twice")),
        }
    }

    pub fn handle(&self) -> Result<&Handle> {
        match &self.runtime {
            Some(rt) => Ok(rt.handle()),
            None => Err(string_error::static_err(
                "can not create handle to completed server",
            )),
        }
    }

    fn wait(&mut self) {
        if let Some(rt) = self.runtime.take() {
            log::debug!(
                "waiting for {:?} to shut down",
                self.config.shutdown_timeout
            );
            rt.shutdown_timeout(self.config.shutdown_timeout);
        }
    }

    async fn serve<F, R>(&self, handler: F) -> Result<()>
    where
        F: Fn(TcpStream) -> R + Sync + 'static,
        R: Future + Send + 'static,
        R::Output: Send + 'static,
    {
        let address = if self.config.public {
            log::warn!("binding port {} publicly to 0.0.0.0", self.config.port);
            format!("0.0.0.0:{}", self.config.port)
        } else {
            log::info!("binding port {} locally to 127.0.0.1", self.config.port);
            format!("127.0.0.1:{}", self.config.port)
        };

        let mut listener = TcpListener::bind(address).await?;

        loop {
            let (stream, remote_address) = listener.accept().await?;
            log::info!("accepted connection from {}", remote_address);

            tokio::spawn(handler(stream));
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.wait();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
