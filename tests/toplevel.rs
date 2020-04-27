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
fn tree_needs_ca_and_hosts() {
    let ca = unique_name("_");
    let host1 = unique_name("_");
    let host2 = unique_name("_");

    let mut cmd = certgen(&["tree"]);
    cmd.unwrap_err();

    let mut cmd = certgen(&["tree", ca.as_str()]);
    cmd.unwrap_err();

    let mut cmd = certgen(&["tree", ca.as_str(), host1.as_str()]);
    cmd.unwrap();

    let mut cmd = certgen(&["tree", ca.as_str(), host1.as_str(), host2.as_str()]);
    cmd.unwrap();
}

#[test]
fn tree_generates_keys_and_certs() {
    let ca = unique_name("sampel-ca");
    let host1 = unique_name("sample-host");
    let host2 = unique_name("sample-host");

    certgen(&["tree", ca.as_str(), host1.as_str(), host2.as_str()])
        .ok()
        .unwrap();

    for name in vec![ca, host1, host2] {
        assert_valid_key(format!("{}-key.pem", name).as_str());
        assert_valid_cert(format!("{}-cert.pem", name).as_str());
    }
}

#[test]
fn sign_needs_request_and_private_key() {
    let ca = unique_name("ca-signer");
    let req = unique_name("request");

    certgen(&["root", ca.as_str()]).ok().unwrap();

    certgen(&["sign"]).ok().unwrap_err();

    certgen(&["sign", ca.as_str()]).ok().unwrap_err();

    certgen(&["sign", ca.as_str(), req.as_str()]).ok().unwrap();
}

#[test]
fn server_config_is_usable() {
    let ca = unique_name("valid-root");
    let host = unique_name("valid-host");
    let client = unique_name("valid-client");

    certgen(&["tree", ca.as_str(), host.as_str(), client.as_str()])
        .ok()
        .unwrap();

    let mut server = make_server(
        format!("{}-key.pem", host).as_str(),
        format!("{}-cert.pem", host).as_str(),
        format!("{}-cert.pem", ca).as_str());

    let mut client = make_client(
        format!("{}-key.pem", client).as_str(),
        format!("{}-cert.pem", client).as_str(),
        format!("{}-cert.pem", ca).as_str(),
        host.as_str());

    assert_eq!(true, client.is_handshaking());

    client
        .complete_io(&mut OtherSession { sess: &mut server })
        .unwrap();
    assert_eq!(false, client.is_handshaking());
}

#[test]
fn client_rejects_bad_ca() {
    let ca = unique_name("valid-root");
    let bad_ca = unique_name("invalid-root");
    let host = unique_name("valid-host");

    certgen(&["tree", bad_ca.as_str(), host.as_str()])
        .ok()
        .unwrap();
    certgen(&["root", ca.as_str()]).ok().unwrap();

    let mut server = make_server(
        format!("{}-key.pem", host).as_str(),
        format!("{}-cert.pem", host).as_str(),
        format!("{}-cert.pem", bad_ca).as_str());

    let mut client = make_client(
        format!("{}-key.pem", host).as_str(),
        format!("{}-cert.pem", host).as_str(),
        format!("{}-cert.pem", ca).as_str(),
        host.as_str());

    assert_eq!(true, client.is_handshaking());

    client
        .complete_io(&mut OtherSession { sess: &mut server })
        .expect_err("should reject");
}

#[test]
fn server_rejects_bad_ca() {
    let ca = unique_name("valid-root");
    let bad_ca = unique_name("invalid-root");
    let host = unique_name("valid-host");
    let client = unique_name("invalid-client");

    certgen(&["tree", ca.as_str(), host.as_str()])
        .ok()
        .unwrap();
    certgen(&["tree", bad_ca.as_str(), client.as_str()]).ok().unwrap();

    let mut server = make_server(
        format!("{}-key.pem", host).as_str(),
        format!("{}-cert.pem", host).as_str(),
        format!("{}-cert.pem", ca).as_str());

    let mut client = make_client(
        format!("{}-key.pem", client).as_str(),
        format!("{}-cert.pem", client).as_str(),
        format!("{}-cert.pem", ca).as_str(),
        host.as_str());

    assert_eq!(true, client.is_handshaking());

    server
        .complete_io(&mut OtherSession { sess: &mut client })
        .expect_err("should reject");
}
