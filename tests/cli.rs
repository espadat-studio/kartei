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
fn exits_2_with_help_on_stderr_without_argument_or_kartei_dir() {
    let output = kartei().output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Usage: kartei"), "{stderr}");
    assert!(stderr.contains("KARTEI_DIR"), "{stderr}");
    assert!(output.stdout.is_empty());
}

#[test]
fn help_prints_usage_to_stdout() {
    for arg in ["--help", "-h", "help"] {
        let output = kartei().arg(arg).output().unwrap();
        assert_eq!(output.status.code(), Some(0), "{arg}");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Usage: kartei"), "{arg}: {stdout}");
        assert!(stdout.contains("KARTEI_DIR"), "{arg}: {stdout}");
    }
}

#[test]
fn exits_2_on_an_unknown_flag() {
    let output = kartei().arg("--bogus").output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("--bogus"));
}

#[test]
fn argument_wins_over_kartei_dir() {
    let output = kartei()
        .env("KARTEI_DIR", "/nonexistent/from-env")
        .arg("/nonexistent/from-arg")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("/nonexistent/from-arg"));
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

#[test]
fn version_flag_prints_name_and_version_to_stdout() {
    for flag in ["--version", "-V"] {
        let output = kartei().arg(flag).output().unwrap();
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            concat!("kartei ", env!("CARGO_PKG_VERSION"), "\n")
        );
    }
}
