use std::path::Path;
use std::sync::mpsc::Sender;
use std::time::Duration;

use notify_debouncer_full::notify::{self, RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{DebounceEventResult, Debouncer, RecommendedCache, new_debouncer};

use crate::vdir;

pub type Watcher = Debouncer<RecommendedWatcher, RecommendedCache>;

const DEBOUNCE: Duration = Duration::from_millis(200);

pub fn watch(book: &Path, changes: Sender<()>) -> notify::Result<Watcher> {
    let file = (!book.is_dir()).then(|| book.to_owned());
    let dir = file
        .as_deref()
        .and_then(Path::parent)
        .unwrap_or(book)
        .to_owned();
    let is_relevant = move |path: &Path| match &file {
        Some(file) => path == file,
        None => vdir::is_vcf(path),
    };
    let mut debouncer = new_debouncer(DEBOUNCE, None, move |result: DebounceEventResult| {
        if let Ok(events) = result
            && events
                .iter()
                .flat_map(|event| &event.paths)
                .any(|path| is_relevant(path))
        {
            let _ = changes.send(());
        }
    })?;
    debouncer.watch(&dir, RecursiveMode::NonRecursive)?;
    Ok(debouncer)
}
