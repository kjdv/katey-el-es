extern crate escargot;

use std::io::{Read, Write};
use std::process::{Child, Stdio};

struct Echo {
    proc: Child,
}

impl Echo {
    fn new(port: u16) -> Echo {
        let port = format!("{}", port);
        let proc = escargot::CargoBuild::new()
            .run()
            .expect("cargo run")
            .command()
            .args(&["--port", port.as_str()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("start echo server");

        Echo { proc }
    }
}

impl Drop for Echo {
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

    fn communicate(&mut self, input: &str) -> String {
        self.stream.write_all(input.as_bytes()).expect("write");

        let mut buf = [0; 4096];
        let size = self.stream.read(&mut buf).expect("read");
        String::from_utf8_lossy(&buf[0..size]).to_string()
    }
}

#[test]
fn single() {
    let _e = Echo::new(3458);
    let mut client = Client::new(3458);

    assert_eq!("foo".to_string(), client.communicate("foo"));
    assert_eq!("bar".to_string(), client.communicate("bar"));
}

#[test]
fn interleaved() {
    let _e = Echo::new(3457);
    let mut client1 = Client::new(3457);
    let mut client2 = Client::new(3457);

    assert_eq!("foo".to_string(), client1.communicate("foo"));
    assert_eq!("bar".to_string(), client2.communicate("bar"));
    assert_eq!("baz".to_string(), client1.communicate("baz"));
    assert_eq!("baz".to_string(), client2.communicate("baz"));
}
