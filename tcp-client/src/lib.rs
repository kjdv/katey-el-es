extern crate futures;
extern crate log;
extern crate string_error;
extern crate tokio;

use futures::future::Future;

pub use tokio::net::TcpStream as Stream;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Copy, Clone)]
pub struct Config<'a> {
    address: &'a str,
    threaded: bool,
    shutdown_timeout: std::time::Duration,
}

impl Config<'_> {
    pub fn new(address: &str) -> Config {
        Config {
            address,
            threaded: false,
            shutdown_timeout: std::time::Duration::from_secs(1),
        }
    }

    pub fn with_threading(&mut self, threaded: bool) -> Self {
        self.threaded = threaded;
        *self
    }

    pub fn with_shutdown_timeout(&mut self, timeout: std::time::Duration) -> Self {
        self.shutdown_timeout = timeout;
        *self
    }
}

pub struct Client<'a> {
    config: Config<'a>,
    runtime: Option<tokio::runtime::Runtime>,
}

impl Client<'_> {
    pub fn new(config: Config) -> Result<Client> {
        log::info!("creating client with configuration {:?}", config);

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
                    let stream = Stream::connect(self.config.address).await?;
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
}

impl Drop for Client<'_> {
    fn drop(&mut self) {
        if let Some(rt) = self.runtime.take() {
            self.wait(rt);
        }
    }
}
