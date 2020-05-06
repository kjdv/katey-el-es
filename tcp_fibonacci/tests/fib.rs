extern crate escargot;

use std::io::Read;
use std::process::{Child, Stdio};

struct Fib {
    proc: Child,
}

impl Fib {
    fn new(port: u16, n: u32) -> Fib {
        let port = format!("{}", port);
        let n = format!("{}", n);
        let proc = escargot::CargoBuild::new()
            .run()
            .expect("cargo run")
            .command()
            .args(&[
                "--port",
                port.as_str(),
                "-n",
                n.as_str(),
                "--interval",
                "0.001",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("start fibonacci server");

        Fib { proc }
    }
}

impl Drop for Fib {
    fn drop(&mut self) {
        let _ = self.proc.kill();
        let _ = self.proc.wait();
    }
}

struct Client {
    stream: std::net::TcpStream,
}

impl Client {
    fn new(port: u16) -> Client {
        let address = format!("127.0.0.1:{}", port);
        let stream = Client::try_connect(&address);
        Client { stream }
    }

    fn try_connect(address: &str) -> std::net::TcpStream {
        for _ in 0..6000 {
            if let Ok(stream) = std::net::TcpStream::connect(address) {
                return stream;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!("failed to connect");
    }

    fn get(&mut self) -> String {
        let mut buf = [0; 4096];
        let size = self.stream.read(&mut buf).expect("read");
        String::from_utf8_lossy(&buf[0..size]).to_string()
    }
}

#[test]
fn single() {
    let _f = Fib::new(3460, 10);
    let mut client = Client::new(3460);

    for i in &[0, 1, 1, 2, 3, 5, 8, 13, 21, 34] {
        let expect = format!("{}\n", i);

        assert_eq!(expect, client.get());
    }
}

#[test]
fn interleaved() {
    let _f = Fib::new(3461, 10);
    let mut client1 = Client::new(3461);
    let mut client2 = Client::new(3461);

    for i in &[0, 1, 1, 2, 3, 5, 8, 13, 21, 34] {
        let expect = format!("{}\n", i);

        assert_eq!(expect, client1.get());
        assert_eq!(expect, client2.get());
    }
}
