extern crate futures;
extern crate log;
extern crate tokio;

use futures::future::{try_select, Either, TryFuture};
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

        if let Err(e) = to.write_all(&buf[0..n]).await {
            log::debug!("write error: {}", e);
            return Err(e);
        }
    }
}

pub async fn select<T, U, E, F, O>(to: T, from: U) -> Result<O>
where
    T: TryFuture<Ok = std::result::Result<O, F>, Error = E> + Unpin,
    U: TryFuture<Ok = std::result::Result<O, F>, Error = E> + Unpin,
    E: std::fmt::Debug + std::convert::Into<Box<dyn std::error::Error>>,
    F: std::fmt::Debug + std::convert::Into<Box<dyn std::error::Error>>,
{
    match try_select(to, from).await {
        Ok(Either::Left((Ok(to), _))) => {
            log::debug!("to->from closed ok");
            Ok(to)
        }
        Ok(Either::Left((Err(to), _))) => {
            log::debug!("to->from closed with error: {:?}", to);
            Err(to.into())
        }
        Ok(Either::Right((Ok(to), _))) => {
            log::debug!("from->to closed ok");
            Ok(to)
        }
        Ok(Either::Right((Err(from), _))) => {
            log::debug!("from->to closed with error: {:?}", from);
            Err(from.into())
        }
        Err(Either::Left((e, _))) => {
            log::debug!("to->from error: {:?}", e);
            Err(e.into())
        }
        Err(Either::Right((e, _))) => {
            log::debug!("from->to error: {:?}", e);
            Err(e.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[tokio::test]
    async fn select_left() {
        let to = tokio::spawn(async {
            let mut reader: &[u8] = b"hello";
            let mut writer: Vec<u8> = vec![];
            copy(&mut reader, &mut writer).await.unwrap();
            let result: std::io::Result<Vec<u8>> = Ok(writer);
            result
        });
        let from = tokio::spawn(async {
            copy(NeverReady {}, tokio::io::sink()).await.unwrap();
            let result: std::io::Result<Vec<u8>> = Ok(vec![]);
            result
        });

        let result: Vec<u8> = select(to, from).await.expect("select");
        assert_eq!(b"hello", result.as_slice());
    }

    #[tokio::test]
    async fn select_right() {
        let from = tokio::spawn(async {
            let mut reader: &[u8] = b"hello";
            let mut writer: Vec<u8> = vec![];
            copy(&mut reader, &mut writer).await.unwrap();
            let result: std::io::Result<Vec<u8>> = Ok(writer);
            result
        });
        let to = tokio::spawn(async {
            copy(NeverReady {}, tokio::io::sink()).await.unwrap();
            let result: std::io::Result<Vec<u8>> = Ok(vec![]);
            result
        });

        let result: Vec<u8> = select(to, from).await.expect("select");
        assert_eq!(b"hello", result.as_slice());
    }

    #[tokio::test]
    async fn select_left_error() {
        let to = tokio::spawn(async {
            let result: std::io::Result<Vec<u8>> =
                Err(std::io::Error::from(std::io::ErrorKind::Other));
            result
        });
        let from = tokio::spawn(async {
            copy(NeverReady {}, tokio::io::sink()).await.unwrap();
            let result: std::io::Result<Vec<u8>> = Ok(vec![]);
            result
        });

        select(to, from).await.expect_err("select");
    }

    #[tokio::test]
    async fn select_right_error() {
        let from = tokio::spawn(async {
            let result: std::io::Result<Vec<u8>> =
                Err(std::io::Error::from(std::io::ErrorKind::Other));
            result
        });
        let to = tokio::spawn(async {
            copy(NeverReady {}, tokio::io::sink()).await.unwrap();
            let result: std::io::Result<Vec<u8>> = Ok(vec![]);
            result
        });

        select(to, from).await.expect_err("select");
    }
}
