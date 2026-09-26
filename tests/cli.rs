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

fn book(test: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("kartei-cli-{test}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let cards = [
        (
            "alan.vcf",
            "BEGIN:VCARD\r\nVERSION:3.0\r\nN:Nunn;Alan;;;\r\nFN:Alan Nunn\r\nEMAIL:alan@n.example\r\nEMAIL;TYPE=home:alan@home.example\r\nEND:VCARD\r\n",
        ),
        (
            "ann.vcf",
            "BEGIN:VCARD\r\nVERSION:3.0\r\nN:Zeller;Ann;;;\r\nFN:Ann Zeller\r\nEMAIL;TYPE=work:ann@z.example\r\nEND:VCARD\r\n",
        ),
        (
            "annabel.vcf",
            "BEGIN:VCARD\r\nVERSION:3.0\r\nN:Nomail;Annabel;;;\r\nFN:Annabel Nomail\r\nTEL:+1 555 0100\r\nEND:VCARD\r\n",
        ),
        (
            "broken.vcf",
            "BEGIN:VCARD\r\nFN:Ann Broken\r\nEMAIL:broken@example.org\r\n",
        ),
    ];
    for (name, text) in cards {
        std::fs::write(dir.join(name), text).unwrap();
    }
    dir
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn query_prints_every_email_of_each_matching_card_ranked() {
    let dir = book("ranked");
    let output = kartei()
        .args(["query", "ann", "--dir"])
        .arg(&dir)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        stdout(&output),
        "ann@z.example\tAnn Zeller\twork\nalan@n.example\tAlan Nunn\t\nalan@home.example\tAlan Nunn\thome\n"
    );
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn query_name_match_prints_every_email_on_that_card() {
    let dir = book("name");
    let output = kartei()
        .args(["query", "nunn", "-d"])
        .arg(&dir)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        stdout(&output),
        "alan@n.example\tAlan Nunn\t\nalan@home.example\tAlan Nunn\thome\n"
    );
}

#[test]
fn query_leaves_out_cards_without_email_and_defective_cards() {
    let dir = book("left-out");
    for text in ["annabel", "broken"] {
        let output = kartei()
            .args(["query", text, "-d"])
            .arg(&dir)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0), "{text}");
        assert_eq!(stdout(&output), "", "{text}");
        assert!(output.stderr.is_empty(), "{text}");
    }
}

#[test]
fn query_mutt_prints_a_header_first() {
    let dir = book("mutt");
    let output = kartei()
        .args(["query", "zeller", "--mutt", "-d"])
        .arg(&dir)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    let stdout = stdout(&output);
    let (header, results) = stdout.split_once('\n').unwrap();
    assert!(!header.contains('\t'), "{header}");
    assert_eq!(results, "ann@z.example\tAnn Zeller\twork\n");
}

#[test]
fn query_mutt_exits_1_with_no_matches() {
    let dir = book("mutt-none");
    let output = kartei()
        .args(["query", "xyzzy", "--mutt", "-d"])
        .arg(&dir)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(stdout(&output).lines().count(), 1);
}

#[test]
fn query_exits_0_with_empty_stdout_on_no_matches() {
    let dir = book("none");
    let output = kartei()
        .args(["query", "xyzzy", "-d"])
        .arg(&dir)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty());
}

#[test]
fn query_dir_wins_over_kartei_dir() {
    let dir = book("dir-wins");
    let output = kartei()
        .env("KARTEI_DIR", "/nonexistent/from-env")
        .args(["query", "zeller", "--dir"])
        .arg(&dir)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "ann@z.example\tAnn Zeller\twork\n");
}

#[test]
fn query_reads_kartei_dir_without_dir() {
    let dir = book("env");
    let output = kartei()
        .env("KARTEI_DIR", &dir)
        .args(["query", "zeller"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "ann@z.example\tAnn Zeller\twork\n");
}

#[test]
fn query_on_a_missing_address_book_writes_stderr_and_exits_1() {
    let output = kartei()
        .args(["query", "ann", "-d", "/nonexistent/kartei"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("/nonexistent/kartei"));
}

#[test]
fn query_without_dir_or_kartei_dir_exits_2() {
    let output = kartei().args(["query", "ann"]).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
}

#[test]
fn abbreviated_query_is_rejected() {
    let output = kartei().args(["que", "x"]).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn a_path_named_query_opens_as_a_path() {
    let dir = book("dot-query");
    let output = kartei().current_dir(&dir).arg("./query").output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("./query"));
}

#[test]
fn help_lists_query() {
    let output = kartei().arg("--help").output().unwrap();
    assert!(stdout(&output).contains("query"));
}

#[test]
fn query_keeps_one_line_per_email_when_fields_hold_newlines_or_tabs() {
    let dir = book("escapes");
    std::fs::write(
        dir.join("escapes.vcf"),
        "BEGIN:VCARD\r\nVERSION:3.0\r\nN:Quux;Tab;;;\r\nFN:Tab\\nQuux\r\nitem1.EMAIL:tab@q.example\r\nitem1.X-ABLabel:a\tb\r\nEND:VCARD\r\n",
    )
    .unwrap();
    let output = kartei()
        .args(["query", "quux", "-d"])
        .arg(&dir)
        .output()
        .unwrap();
    assert_eq!(stdout(&output), "tab@q.example\tTab Quux\ta b\n");
}
