mod common;
use common::*;
use rustls::Session;

#[test]
fn subcommand_is_mandatory() {
    let mut cmd = certgen(&[]);
    cmd.unwrap_err();
}

#[test]
fn root_needs_name_arg() {
    let mut cmd = certgen(&["root"]);
    cmd.unwrap_err();

    let mut cmd = certgen(&["root", "_"]);
    cmd.unwrap();
}

#[test]
fn root_generates_key_and_ca() {
    certgen(&["root", "sample-root"]).ok().unwrap();

    assert_valid_key("sample-root-key.pem");
    assert_valid_cert("sample-root-ca.pem");
}

#[test]
fn server_config_is_usable() {
    certgen(&["root", "valid-root"]).ok().unwrap();

    let (mut client, mut server) = make_pair(
        "valid-root-key.pem",
        "valid-root-ca.pem",
        "valid-root-ca.pem",
        "valid-root",
    );
    assert_eq!(true, client.is_handshaking());

    client
        .complete_io(&mut OtherSession { sess: &mut server })
        .unwrap();
    assert_eq!(false, client.is_handshaking());
}

#[test]
fn client_rejects_bad_cert() {
    certgen(&["root", "invalid-root-1"]).ok().unwrap();
    certgen(&["root", "invalid-root-2"]).ok().unwrap();

    let (mut client, mut server) = make_pair(
        "invalid-root-1-key.pem",
        "invalid-root-1-ca.pem",
        "invalid-root-2-ca.pem",
        "invalid-root-1",
    );
    assert_eq!(true, client.is_handshaking());

    let mut other = OtherSession { sess: &mut server };
    client.complete_io(&mut other).expect_err("should reject");
}
