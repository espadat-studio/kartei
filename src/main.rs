use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use clap::{Parser, Subcommand};
use kartei::app::App;
use kartei::vdir::AddressBook;
use kartei::{editor, query, ui, vdir, watch};
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{self, EnterAlternateScreen};

const TICK: Duration = Duration::from_millis(100);

#[derive(Parser)]
#[command(
    version,
    about,
    arg_required_else_help = true,
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true
)]
struct Cli {
    /// Address book: a directory of .vcf files, or one .vcf file
    #[arg(env = "KARTEI_DIR", value_name = "DIR_OR_FILE.VCF", required = true)]
    path: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Print emails of matching cards for mail client address completion
    Query {
        /// Text to match, as in / search
        text: String,
        /// Address book: a directory of .vcf files, or one .vcf file
        #[arg(short, long, env = "KARTEI_DIR", value_name = "DIR_OR_FILE.VCF")]
        dir: PathBuf,
        /// Print mutt's header line first and exit 1 on no match
        #[arg(long)]
        mutt: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Query { text, dir, mutt }) => query(&dir, &text, mutt),
        None => tui(&cli.path.expect("clap requires a path without a command")),
    }
}

fn load(path: &Path) -> Option<AddressBook> {
    match std::path::absolute(path).and_then(|path| vdir::load(&path)) {
        Ok(book) => Some(book),
        Err(err) => {
            eprintln!("kartei: {}: {err}", path.display());
            None
        }
    }
}

fn query(path: &Path, text: &str, mutt: bool) -> ExitCode {
    let Some(book) = load(path) else {
        return ExitCode::FAILURE;
    };
    let lines = query::lines(&book, text);
    let mut out = String::new();
    if mutt {
        out.push_str(&format!("kartei: {} matches\n", lines.len()));
    }
    for line in &lines {
        out.push_str(line);
        out.push('\n');
    }
    match io::stdout().lock().write_all(out.as_bytes()) {
        Err(err) if err.kind() != io::ErrorKind::BrokenPipe => {
            eprintln!("kartei: {err}");
            ExitCode::FAILURE
        }
        _ if mutt && lines.is_empty() => ExitCode::FAILURE,
        _ => ExitCode::SUCCESS,
    }
}

fn tui(path: &Path) -> ExitCode {
    let Some(book) = load(path) else {
        return ExitCode::FAILURE;
    };
    let (changes, changed) = mpsc::channel();
    let _watcher = match watch::watch(&book.path, changes) {
        Ok(watcher) => watcher,
        Err(err) => {
            eprintln!("kartei: {}: watch failed: {err}", path.display());
            return ExitCode::FAILURE;
        }
    };
    let result = run(&mut ratatui::init(), App::new(book), &changed);
    ratatui::restore();
    if let Err(err) = result {
        eprintln!("kartei: {err}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run(terminal: &mut DefaultTerminal, mut app: App, changed: &Receiver<()>) -> io::Result<()> {
    while !app.should_quit() {
        terminal.draw(|frame| ui::draw(frame, &app))?;
        handle_next_event(&mut app, changed)?;
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

fn handle_next_event(app: &mut App, changed: &Receiver<()>) -> io::Result<()> {
    loop {
        if event::poll(TICK)? {
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                app.handle_key(key);
            }
            return Ok(());
        }
        if changed.try_iter().count() > 0 {
            app.disk_changed();
            return Ok(());
        }
    }
}
