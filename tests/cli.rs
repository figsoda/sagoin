use std::process::Command;

#[test]
fn basic() {
    assert_success(&["-h"]);
    assert_success(&["--help"]);
    assert_success(&["-V"]);
    assert_success(&["--version"]);
}

fn assert_success(args: &[&str]) {
    let res = Command::new(env!("CARGO_BIN_EXE_sagoin"))
        .args(args)
        .output()
        .unwrap();
    assert!(res.status.success(), "{:#?}", res);
}
