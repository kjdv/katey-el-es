extern crate escargot;
extern crate tempfile;

use std::io::{BufRead, Read, Write};
use std::process::{Child, Stdio};

pub struct Fixture {
    tempdir: tempfile::TempDir,

    tcp_echo: Child,
    echo_port: u16,

    tcp_fib: Child,
    fib_port: u16,

    tls_echo: Child,
    tls_echo_port: u16,

    tls_fib: Child,
    tls_fib_port: u16,
}

impl Fixture {
    pub fn new(base_port: u16) -> Fixture {
        let tempdir = tempfile::TempDir::new().unwrap();

        let echo_port = base_port;
        let fib_port = base_port + 1;
        let tls_echo_port = base_port + 2;
        let tls_fib_port = base_port + 3;

        certgen(tempdir.path(), "this-root", &["this-server", "this-client"]);
        certgen(tempdir.path(), "other-root", &["other-client"]);

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
        let tls_echo = escargot::CargoBuild::new()
            .manifest_path(manifest())
            .bin("katey-el-es")
            .run()
            .expect("cargo run")
            .command()
            .arg(format!("{}", tls_echo_port))
            .arg(format!("127.0.0.1:{}", echo_port))
            .arg("--cert")
            .arg(certfile(tempdir.path(), "this-server"))
            .arg("--key")
            .arg(keyfile(tempdir.path(), "this-server"))
            .stdout(Stdio::null())
            .spawn()
            .expect("spawn");
        let tls_fib = escargot::CargoBuild::new()
            .manifest_path(manifest())
            .bin("katey-el-es")
            .run()
            .expect("cargo run")
            .command()
            .arg(format!("{}", tls_fib_port))
            .arg(format!("127.0.0.1:{}", fib_port))
            .arg("--cert")
            .arg(certfile(tempdir.path(), "this-server"))
            .arg("--key")
            .arg(keyfile(tempdir.path(), "this-server"))
            .arg("--authenticate")
            .arg(certfile(tempdir.path(), "this-root"))
            .stdout(Stdio::null())
            .spawn()
            .expect("spawn");

        for port in &[echo_port, fib_port, tls_echo_port, tls_fib_port] {
            wait_for(*port, 1.0).expect("port");
        }

        Fixture {
            tempdir,
            tcp_echo,
            echo_port,
            tcp_fib,
            fib_port,
            tls_echo,
            tls_echo_port,
            tls_fib,
            tls_fib_port,
        }
    }

    pub fn tcp_echo_client(&self) -> Client {
        Fixture::tcp_client(self.echo_port)
    }

    pub fn tcp_fibonacci_client(&self) -> Client {
        Fixture::tcp_client(self.fib_port)
    }

    pub fn tls_echo_client(&self, root: &str) -> Client {
        self.tls_client(self.tls_echo_port, root, None)
    }

    pub fn tls_fib_client(&self, root: &str, name: &str) -> Client {
        self.tls_client(self.tls_fib_port, root, Some(name))
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
            .stderr(Stdio::null())
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

    fn tls_client(&self, port: u16, root: &str, name: Option<&str>) -> Client {
        let args: Vec<String> = if let Some(name) = name {
            vec![
                "--key".to_string(),
                keyfile(self.tempdir.path(), name),
                "--cert".to_string(),
                certfile(self.tempdir.path(), name),
            ]
        } else {
            vec![]
        };

        let mut process = escargot::CargoBuild::new()
            .manifest_path(manifest())
            .bin("katey-client")
            .run()
            .expect("cargo run")
            .command()
            .arg(format!("localhost:{}", port))
            .arg("--root")
            .arg(certfile(self.tempdir.path(), root))
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
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
        let _ = self.tls_fib.kill();
        let _ = self.tls_fib.wait();

        let _ = self.tls_echo.kill();
        let _ = self.tls_echo.wait();

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

    pub fn assert_rejected(&mut self) {
        let _ = self
            .writer
            .write_all(b"ping\n")
            .and_then(|_| self.writer.flush())
            .and_then(|_| {
                let mut buf = [0; 1];
                self.reader.read(&mut buf)
            })
            .map(|n| {
                assert_eq!(0, n);
            });

        self.process.wait().unwrap();
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

fn certgen(dir: &std::path::Path, root: &str, children: &[&str]) {
    escargot::CargoBuild::new()
        .manifest_path(manifest())
        .bin("certgen")
        .run()
        .expect("cargo run")
        .command()
        .current_dir(dir)
        .arg("tree")
        .arg(root)
        .args(children)
        .stdout(Stdio::piped())
        .output()
        .expect("spawn");
}

fn certfile(dir: &std::path::Path, name: &str) -> String {
    format!("{}/{}-cert.pem", dir.as_os_str().to_str().unwrap(), name)
}

fn keyfile(dir: &std::path::Path, name: &str) -> String {
    format!("{}/{}-key.pem", dir.as_os_str().to_str().unwrap(), name)
}
