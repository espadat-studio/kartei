use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use kartei::app::App;
use kartei::{ui, vdir};
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyEventKind};

fn main() -> ExitCode {
    let Some(path) = env::args_os().nth(1).or_else(|| env::var_os("KARTEI_DIR")) else {
        eprintln!("usage: kartei <address-book-dir-or-vcf-file> (or set KARTEI_DIR)");
        return ExitCode::FAILURE;
    };
    let path = PathBuf::from(path);
    let book = match vdir::load(&path) {
        Ok(book) => book,
        Err(err) => {
            eprintln!("kartei: {}: {err}", path.display());
            return ExitCode::FAILURE;
        }
    };

    let result = run(&mut ratatui::init(), App::new(book));
    ratatui::restore();
    if let Err(err) = result {
        eprintln!("kartei: {err}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run(terminal: &mut DefaultTerminal, mut app: App) -> io::Result<()> {
    while !app.should_quit() {
        terminal.draw(|frame| ui::draw(frame, &app))?;
        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.handle_key(key);
        }
        if let Some(sequence) = app.take_clipboard() {
            terminal.backend_mut().write_all(sequence.as_bytes())?;
            terminal.backend_mut().flush()?;
        }
    }
    Ok(())
}
