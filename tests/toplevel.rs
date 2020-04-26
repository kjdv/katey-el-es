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
    let name = unique_name("_");

    let mut cmd = certgen(&["root"]);
    cmd.unwrap_err();

    let mut cmd = certgen(&["root", name.as_str()]);
    cmd.unwrap();
}

#[test]
fn root_generates_key_and_ca() {
    let name = unique_name("sample-root");
    certgen(&["root", name.as_str()]).ok().unwrap();

    assert_valid_key(format!("{}-key.pem", name).as_str());
    assert_valid_cert(format!("{}-cert.pem", name).as_str());
}

#[test]
fn request_needs_name_arg() {
    let name = unique_name("_");

    let mut cmd = certgen(&["request"]);
    cmd.unwrap_err();

    let mut cmd = certgen(&["request", name.as_str()]);
    cmd.unwrap();
}

#[test]
fn request_generates_key_and_ca() {
    let name = unique_name("sample-request");
    certgen(&["request", name.as_str()]).ok().unwrap();

    assert_valid_key(format!("{}-key.pem", name).as_str());
    assert_valid_request(format!("{}-request.pem", name).as_str());
}

#[test]
fn sign_needs_request_and_private_key() {
    let ca = unique_name("ca-signer");
    let req = unique_name("request");

    certgen(&["root", ca.as_str()]).ok().unwrap();
    certgen(&["request", req.as_str()]).ok().unwrap();

    certgen(&["sign"]).ok().unwrap_err();

    certgen(&["sign", req.as_str()]).ok().unwrap_err();

    certgen(&["sign", req.as_str(), ca.as_str()]).ok().unwrap();
}

#[test]
fn server_config_is_usable() {
    let name = unique_name("valid-root");

    certgen(&["root", name.as_str()]).ok().unwrap();

    let (mut client, mut server) = make_pair(
        format!("{}-key.pem", name).as_str(),
        format!("{}-cert.pem", name).as_str(),
        format!("{}-cert.pem", name).as_str(),
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
        format!("{}-cert.pem", name1).as_str(),
        format!("{}-cert.pem", name2).as_str(),
        name2.as_str(),
    );
    assert_eq!(true, client.is_handshaking());

    let mut other = OtherSession { sess: &mut server };
    client.complete_io(&mut other).expect_err("should reject");
}
