use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use kartei::app::App;
use kartei::{editor, ui, vdir};
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{self, EnterAlternateScreen};

#[derive(Parser)]
#[command(version, about, arg_required_else_help = true)]
struct Cli {
    /// Address book: a directory of .vcf files, or one .vcf file
    #[arg(env = "KARTEI_DIR", value_name = "DIR_OR_FILE.VCF")]
    path: PathBuf,
}

fn main() -> ExitCode {
    let mut args: Vec<_> = env::args_os().collect();
    if args.get(1).is_some_and(|arg| arg == "help") {
        args[1] = "--help".into();
    }
    let path = Cli::parse_from(args).path;
    let book = match std::path::absolute(&path).and_then(|path| vdir::load(&path)) {
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
        if let Some(bytes) = app.take_editor_request() {
            ratatui::restore();
            let edited = editor::run(&bytes);
            terminal::enable_raw_mode()?;
            execute!(io::stdout(), EnterAlternateScreen)?;
            terminal.clear()?;
            app.finish_raw_edit(edited);
        }
    }
    Ok(())
}
