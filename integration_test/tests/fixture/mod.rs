extern crate escargot;

use std::io::{BufRead, Write};
use std::process::{Child, Stdio};

pub struct Fixture {
    tcp_echo: Child,
    echo_port: u16,

    tcp_fib: Child,
    fib_port: u16,
}

impl Fixture {
    pub fn new(base_port: u16) -> Fixture {
        let echo_port = base_port;
        let fib_port = base_port + 1;

        let tcp_echo = escargot::CargoBuild::new()
            .manifest_path(manifest())
            .bin("tcp_echo")
            .run()
            .expect("cargo run")
            .command()
            .arg(format!("--port={}", base_port))
            .stdout(Stdio::null())
            .spawn()
            .expect("spawn");
        let tcp_fib = escargot::CargoBuild::new()
            .manifest_path(manifest())
            .bin("tcp_fibonacci")
            .run()
            .expect("cargo run")
            .command()
            .arg(format!("--port={}", fib_port))
            .arg("--number=10")
            .arg("--interval=0.01")
            .stdout(Stdio::null())
            .spawn()
            .expect("spawn");

        for port in &[echo_port, fib_port] {
            wait_for(*port, 1.0).expect("port");
        }

        Fixture { tcp_echo, echo_port, tcp_fib, fib_port}
    }

    pub fn tcp_echo_client(&self) -> Client {
        Fixture::tcp_client(self.echo_port)
    }

    pub fn tcp_fibonacci_client(&self) -> Client {
        Fixture::tcp_client(self.fib_port)
    }

    fn tcp_client(port: u16) -> Client {
        let mut process = escargot::CargoBuild::new()
            .manifest_path(manifest())
            .bin("tcp_client")
            .run()
            .expect("cargo run")
            .command()
            .arg(format!("127.0.0.1:{}", port))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("spawn");

        let reader = std::io::BufReader::new(process.stdout.take().unwrap());
        let writer = std::io::BufWriter::new(process.stdin.take().unwrap());

        Client {
            process,
            reader,
            writer,
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self.tcp_fib.kill();
        let _ = self.tcp_fib.wait();

        let _ = self.tcp_echo.kill();
        let _ = self.tcp_echo.wait();
    }
}

pub struct Client {
    pub process: Child,
    pub reader: std::io::BufReader<std::process::ChildStdout>,
    pub writer: std::io::BufWriter<std::process::ChildStdin>,
}

impl Client {
    pub fn assert_can_echo(&mut self) {
        self.assert_can_echo_line(b"foo\n");
        self.assert_can_echo_line(b"bar\n");
    }

    pub fn assert_can_listen(&mut self) {
        for i in &[0, 1, 1, 2, 3, 5, 8, 13, 21, 34] {
            let expect = format!("{}\n", i);

            let mut buf = String::new();
            self.reader.read_line(&mut buf).expect("read");
            assert_eq!(expect, buf);
        }
    }

    fn assert_can_echo_line(&mut self, line: &[u8]) {
        self.writer.write_all(line).unwrap();
        self.writer.flush().unwrap();

        let mut buf = String::new();
        self.reader.read_line(&mut buf).expect("read");
        assert_eq!(line, buf.as_bytes());
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

fn wait_for(port: u16, timeout: f64) -> std::result::Result<(), &'static str> {
    use std::time::{Duration, Instant};

    let address = format!("127.0.0.1:{}", port);
    let end = Instant::now() + Duration::from_secs_f64(timeout);

    while Instant::now() <= end {
        if std::net::TcpStream::connect(&address).is_ok() {
            return Ok(());
        }
    }
    Err("timeout")
}

fn manifest() -> std::path::PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let this_dir = std::path::PathBuf::from(manifest_dir);
    let mut parent = this_dir.parent().unwrap().to_path_buf();
    parent.push("Cargo.toml");
    parent
}
