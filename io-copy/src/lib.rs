extern crate log;
extern crate tokio;
extern crate futures;

use std::marker::Unpin;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use futures::future::{try_select, Either, TryFuture};


const BUFSIZE: usize = 512;

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

pub async fn select<T, U, E>(to: T, from: U) -> std::result::Result<(), Box<dyn std::error::Error>>
    where T: TryFuture<Ok=std::io::Result<()>, Error=E> + Unpin,
          U: TryFuture<Ok=std::io::Result<()>, Error=E> + Unpin,
          E: std::fmt::Debug + std::convert::Into<Box<dyn std::error::Error>>
{
    match try_select(to, from).await {
        Ok(Either::Left((Ok(_), _))) => {
            log::debug!("to->from closed ok");
            Ok(())
        }
        Ok(Either::Left((Err(to), _))) => {
            log::debug!("to->from closed with error: {:?}", to);
            Err(to.into())
        }
        Ok(Either::Right((Ok(_), _))) => {
            log::debug!("from->to closed ok");
            Ok(())
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
}
