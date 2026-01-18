mod app;
mod cache;
mod github;
mod input;
mod repo;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use app::App;

fn main() -> Result<()> {
    let (owner, repo_name) = match repo::current_repo() {
        Ok(repo) => repo,
        Err(err) => {
            eprintln!("Repo error: {err}");
            return Ok(());
        }
    };

    let issues = match cache::read_issues() {
        Ok(cached) => cached,
        Err(_) => match github::fetch_open_issues(&owner, &repo_name) {
            Ok(list) => {
                let _ = cache::write_issues(&list);
                list
            }
            Err(err) => {
                eprintln!("Issue error: {err}");
                return Ok(());
            }
        },
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    app.issues = issues;

    let result = run_app(&mut terminal, &mut app, &owner, &repo_name);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {err}");
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    owner: &str,
    repo_name: &str,
) -> io::Result<()> {
    loop {
        ensure_detail(app, owner, repo_name);
        if let Ok(size) = terminal.size() {
            let preview_height = size.height.saturating_sub(2) as usize;
            app.preview_max_lines = app.preview_max_lines.max(preview_height);
            app.preview_page_size = preview_height;
        }
        terminal.draw(|frame| {
            ui::draw(frame, app);
        })?;

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    input::handle_key_event(app, key);
                }
                _ => {}
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn ensure_detail(app: &mut App, owner: &str, repo_name: &str) {
    let Some(number) = app.selected_issue_number() else { return; };
    if app.last_attempted == Some(number) {
        return;
    }
    if app.detail_cache.contains_key(&number) {
        app.loading = None;
        return;
    }

    if let Ok(detail) = cache::read_detail(number) {
        app.detail_cache.insert(number, detail);
        app.loading = None;
        return;
    }

    app.last_attempted = Some(number);
    app.loading = Some(number);
    match github::fetch_issue_detail(owner, repo_name, number) {
        Ok(detail) => {
            let _ = cache::write_detail(number, &detail);
            app.detail_cache.insert(number, detail);
            app.loading = None;
        }
        Err(err) => {
            app.set_error(err.to_string());
            app.loading = None;
        }
    }
}
