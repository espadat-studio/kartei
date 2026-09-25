use std::process::Command;

fn kartei() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_kartei"));
    command.env_remove("KARTEI_DIR");
    command
}

#[test]
fn exits_1_on_missing_path() {
    let output = kartei().arg("/nonexistent/kartei").output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("/nonexistent/kartei"));
}

#[test]
fn exits_1_on_a_path_that_is_neither_a_directory_nor_a_vcf_file() {
    let output = kartei().arg(env!("CARGO_MANIFEST_PATH")).output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("not a directory or .vcf file"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn exits_1_without_argument_or_kartei_dir() {
    let output = kartei().output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("KARTEI_DIR"));
}

#[test]
fn reads_kartei_dir_when_no_argument() {
    let output = kartei()
        .env("KARTEI_DIR", "/nonexistent/from-env")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("/nonexistent/from-env"));
}

#[test]
fn a_file_path_that_is_not_vcf_is_rejected_from_kartei_dir_too() {
    let output = kartei()
        .env("KARTEI_DIR", env!("CARGO_MANIFEST_PATH"))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("not a directory or .vcf file"));
}
