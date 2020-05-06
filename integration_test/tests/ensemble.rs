mod fixture;

use fixture::Fixture;

#[test]
fn tcp_echo() {
    let fix = Fixture::new(4000);

    let mut client = fix.tcp_echo_client();
    client.assert_can_echo();
}

#[test]
fn tcp_fibonacci() {
    let fix = Fixture::new(4004);

    let mut client = fix.tcp_fibonacci_client();
    client.assert_can_listen();
}
