use std::fs;
use std::sync::Mutex;

use kartei::editor;

static ENV: Mutex<()> = Mutex::new(());

#[test]
fn runs_visual_on_a_private_temp_file_and_returns_the_edited_bytes() {
    let _env = ENV.lock().unwrap();
    let dir = std::env::temp_dir().join(format!("kartei-editor-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let log = dir.join("log");
    let visual = format!(
        "f() {{ stat -c %a \"$1\" > '{0}'; echo \"$1\" >> '{0}'; sed -i s/Anna/Hanna/ \"$1\"; }}; f",
        log.display()
    );
    unsafe {
        std::env::set_var("VISUAL", visual);
        std::env::set_var("EDITOR", "false");
    }
    let edited = editor::run(b"FN:Anna Adams\n").unwrap();
    assert_eq!(edited, b"FN:Hanna Adams\n");
    let log = fs::read_to_string(log).unwrap();
    let (mode, path) = log.trim_end().split_once('\n').unwrap();
    assert_eq!(mode, "600");
    assert!(
        path.starts_with(std::env::temp_dir().to_str().unwrap()),
        "{path}"
    );
    assert!(!fs::exists(path).unwrap(), "{path} left behind");
}

#[test]
fn fails_when_neither_visual_nor_editor_is_set() {
    let _env = ENV.lock().unwrap();
    unsafe {
        std::env::remove_var("VISUAL");
        std::env::remove_var("EDITOR");
    }
    let err = editor::run(b"FN:Anna Adams\n").unwrap_err();
    assert!(err.to_string().contains("$VISUAL"), "{err}");
    assert!(err.to_string().contains("$EDITOR"), "{err}");
}
