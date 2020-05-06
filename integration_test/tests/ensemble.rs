mod fixture;

use fixture::Fixture;

const BASE_PORT: u16 = 4000;
const PORT_STEP: u16 = 6;

fn base_port(num: u16) -> u16 {
    BASE_PORT + num * PORT_STEP
}

#[test]
fn tcp_echo() {
    let fix = Fixture::new(base_port(0));

    let mut client = fix.tcp_echo_client();
    client.assert_can_echo();
}

#[test]
fn tcp_fibonacci() {
    let fix = Fixture::new(base_port(1));

    let mut client = fix.tcp_fibonacci_client();
    client.assert_can_listen();
}

#[test]
fn tls_echo() {
    let fix = Fixture::new(base_port(2));

    let mut client = fix.tls_echo_client("this-root");
    client.assert_can_echo();
}

#[test]
fn tls_rejects() {
    let fix = Fixture::new(base_port(3));

    let mut client = fix.tls_echo_client("other-root");
    client.assert_rejected();
}

#[test]
fn tls_fib() {
    let fix = Fixture::new(base_port(4));

    let mut client = fix.tls_fib_client("this-root", "this-client");
    client.assert_can_listen();
}


#[test]
fn tls_reject_client() {
    let fix = Fixture::new(base_port(5));

    let mut client = fix.tls_fib_client("this-root", "other-client");
    client.assert_rejected();
}
