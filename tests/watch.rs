use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use kartei::watch;

const TIMEOUT: Duration = Duration::from_secs(5);
const QUIET: Duration = Duration::from_secs(1);

fn temp_dir(test: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("kartei-{test}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn reports_a_vcf_written_and_renamed_into_a_directory() {
    let dir = temp_dir("watch-dir");
    let (tx, rx) = mpsc::channel();
    let _watcher = watch::watch(&dir, tx).unwrap();

    fs::write(dir.join("anna.vcf.tmp"), "BEGIN:VCARD\r\n").unwrap();
    fs::rename(dir.join("anna.vcf.tmp"), dir.join("anna.vcf")).unwrap();
    rx.recv_timeout(TIMEOUT).unwrap();
}

#[test]
fn reports_a_rename_over_a_single_file_and_ignores_its_siblings() {
    let dir = temp_dir("watch-file");
    let book = dir.join("Contacts.vcf");
    fs::write(&book, "BEGIN:VCARD\r\n").unwrap();
    let (tx, rx) = mpsc::channel();
    let _watcher = watch::watch(&book, tx).unwrap();

    fs::write(dir.join("other.vcf"), "BEGIN:VCARD\r\n").unwrap();
    assert!(rx.recv_timeout(QUIET).is_err(), "sibling reported");

    fs::write(dir.join("Contacts.vcf.tmp"), "BEGIN:VCARD\r\nEND:VCARD\r\n").unwrap();
    fs::rename(dir.join("Contacts.vcf.tmp"), &book).unwrap();
    rx.recv_timeout(TIMEOUT).unwrap();
}
