use std::env;
use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::process::Command;

use uuid::Uuid;

pub fn run(bytes: &[u8]) -> io::Result<Vec<u8>> {
    let editor = ["VISUAL", "EDITOR"]
        .into_iter()
        .filter_map(env::var_os)
        .find(|editor| !editor.is_empty())
        .ok_or_else(|| io::Error::other("neither $VISUAL nor $EDITOR is set"))?;
    let path = env::temp_dir().join(format!("kartei-{}.vcf", Uuid::new_v4()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)?;
    let edited = file.write_all(bytes).and_then(|()| {
        drop(file);
        edit(editor, &path)
    });
    let removed = match fs::remove_file(&path) {
        Err(err) if err.kind() != io::ErrorKind::NotFound => Err(io::Error::new(
            err.kind(),
            format!("{} not deleted: {err}", path.display()),
        )),
        _ => Ok(()),
    };
    let edited = edited?;
    removed?;
    Ok(edited)
}

fn edit(editor: OsString, path: &Path) -> io::Result<Vec<u8>> {
    let mut script = editor.clone();
    script.push(" \"$@\"");
    let status = Command::new("sh")
        .arg("-c")
        .arg(script)
        .arg(editor)
        .arg(path)
        .status()?;
    if !status.success() {
        return Err(io::Error::other(format!("editor {status}")));
    }
    fs::read(path)
}
