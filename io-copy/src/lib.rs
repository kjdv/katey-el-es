extern crate log;
extern crate tokio;

use std::marker::Unpin;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

type Result = std::io::Result<()>;

const BUFSIZE: usize = 512;

pub async fn copy<T, U>(mut from: T, mut to: U) -> Result
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
