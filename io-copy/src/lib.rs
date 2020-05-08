extern crate log;
extern crate tokio;

use std::marker::Unpin;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const BUFSIZE: usize = 512;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub async fn copy<T, U>(mut from: T, mut to: U) -> std::io::Result<()>
where
    T: AsyncReadExt + Unpin,
    U: AsyncWriteExt + Unpin,
{
    let mut buf = [0; BUFSIZE];
    loop {
        let n = match from.read(&mut buf).await {
            Err(e) => {
                log::debug!("read error: {}", e);
                return Err(e);
            }
            Ok(0) => {
                log::debug!("0 read");
                return Ok(());
            }
            Ok(n) => n,
        };

        log::debug!("transferring {} bytes", n);

        if let Err(e) = to.write_all(&buf[0..n]).await {
            log::debug!("write error: {}", e);
            return Err(e);
        }
    }
}

pub async fn proxy<T, U, V, W>(stream1: (T, U), stream2: (V, W)) -> Result<()>
where
    T: AsyncReadExt + Unpin,
    U: AsyncWriteExt + Unpin,
    V: AsyncReadExt + Unpin,
    W: AsyncWriteExt + Unpin,
{
    let (rx1, tx1) = stream1;
    let (rx2, tx2) = stream2;

    // Q: select or join?
    tokio::select! {
        x = copy(rx1, tx2) => {
            match x {
                Ok(_) => {
                    log::info!("rx1->tx2 completed");
                    Ok(())
                },
                Err(e) => {
                    log::warn!("rx1->tx2 errored: {}", e);
                    Err(e.into())
                }
            }
        },
        x = copy(rx2, tx1) => {
            match x {
                Ok(_) => {
                    log::info!("rx2->tx1 completed");
                    Ok(())
                },
                Err(e) => {
                    log::warn!("rx2->tx1 errored: {}", e);
                    Err(e.into())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::sink;

    #[tokio::test]
    async fn basic() {
        let mut reader: &[u8] = b"hello";
        let mut writer: Vec<u8> = vec![];

        copy(&mut reader, &mut writer).await.expect("copy");

        assert_eq!(b"hello", writer.as_slice());
    }

    #[tokio::test]
    async fn large() {
        let mut reader: Vec<u8> = Vec::new();
        while reader.len() <= BUFSIZE {
            reader.extend_from_slice(b"0123456789");
        }
        let mut writer: Vec<u8> = vec![];
        let expect = reader.clone();

        copy(&mut reader.as_slice(), &mut writer)
            .await
            .expect("copy");

        assert_eq!(expect, writer);
    }

    struct NeverReady {}

    impl tokio::io::AsyncRead for NeverReady {
        fn poll_read(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
            _buf: &mut [u8],
        ) -> std::task::Poll<std::io::Result<usize>> {
            std::task::Poll::Pending
        }
    }

    struct AlwaysBad {}

    impl tokio::io::AsyncRead for AlwaysBad {
        fn poll_read(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
            _buf: &mut [u8],
        ) -> std::task::Poll<std::io::Result<usize>> {
            let err = std::io::Error::from(std::io::ErrorKind::Other);
            std::task::Poll::Ready(Err(err))
        }
    }

    #[tokio::test]
    async fn proxy_left() {
        let mut reader: &[u8] = b"hello";
        let mut writer: Vec<u8> = vec![];

        proxy((&mut reader, sink()), (NeverReady {}, &mut writer))
            .await
            .unwrap();

        assert_eq!(b"hello", writer.as_slice());
    }

    #[tokio::test]
    async fn proxy_right() {
        let mut reader: &[u8] = b"hello";
        let mut writer: Vec<u8> = vec![];

        proxy((NeverReady {}, &mut writer), (&mut reader, sink()))
            .await
            .unwrap();

        assert_eq!(b"hello", writer.as_slice());
    }

    #[tokio::test]
    async fn proxy_left_error() {
        proxy((AlwaysBad {}, sink()), (NeverReady {}, sink()))
            .await
            .expect_err("err");
    }

    #[tokio::test]
    async fn proxy_right_error() {
        proxy((NeverReady {}, sink()), (AlwaysBad {}, sink()))
            .await
            .expect_err("err");
    }
}
