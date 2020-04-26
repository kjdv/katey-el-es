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
    let name = unique_name("sample-root");
    certgen(&["root", name.as_str()]).ok().unwrap();

    assert_valid_key(format!("{}-key.pem", name).as_str());
    assert_valid_cert(format!("{}-ca.pem", name).as_str());
}

#[test]
fn server_config_is_usable() {
    let name = unique_name("valid-root");

    certgen(&["root", name.as_str()]).ok().unwrap();

    let (mut client, mut server) = make_pair(
        format!("{}-key.pem", name).as_str(),
        format!("{}-ca.pem", name).as_str(),
        format!("{}-ca.pem", name).as_str(),
        name.as_str(),
    );
    assert_eq!(true, client.is_handshaking());

    client
        .complete_io(&mut OtherSession { sess: &mut server })
        .unwrap();
    assert_eq!(false, client.is_handshaking());
}

#[test]
fn client_rejects_bad_cert() {
    let name1 = unique_name("invalid-root-1");
    let name2 = unique_name("invalid-root-2");

    certgen(&["root", name1.as_str()]).ok().unwrap();
    certgen(&["root", name2.as_str()]).ok().unwrap();

    let (mut client, mut server) = make_pair(
        format!("{}-key.pem", name1).as_str(),
        format!("{}-ca.pem", name1).as_str(),
        format!("{}-ca.pem", name2).as_str(),
        name2.as_str(),
    );
    assert_eq!(true, client.is_handshaking());

    let mut other = OtherSession { sess: &mut server };
    client.complete_io(&mut other).expect_err("should reject");
}
