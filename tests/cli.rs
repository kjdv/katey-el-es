extern crate assert_cmd;

fn certgen(args: &[&str]) -> assert_cmd::Command {
    let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .expect("cargo bin");
    cmd.args(args);
    cmd.current_dir(std::env::current_exe().unwrap().parent().unwrap());
    cmd
}

#[test]
fn subcommand_is_mandatory() {
    let mut cmd = certgen(&[]);
    cmd.unwrap_err();
}
