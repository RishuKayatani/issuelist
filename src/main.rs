mod app;
mod cache;
mod github;
mod input;
mod repo;
mod ui;

use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use app::App;


#[derive(Debug)]
enum WorkerReq {
    FetchList { owner: String, repo: String },
    FetchDetail { owner: String, repo: String, number: u64 },
}

#[derive(Debug)]
enum WorkerMsg {
    ListReady { issues: Vec<github::Issue> },
    DetailReady { number: u64, detail: github::IssueDetail },
    Error { message: String },
}

fn spawn_worker(tx: mpsc::Sender<WorkerMsg>, rx: mpsc::Receiver<WorkerReq>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for req in rx {
            match req {
                WorkerReq::FetchList { owner, repo } => {
                    match github::fetch_open_issues(&owner, &repo) {
                        Ok(list) => {
                            let _ = tx.send(WorkerMsg::ListReady { issues: list });
                        }
                        Err(err) => {
                            let _ = tx.send(WorkerMsg::Error { message: err.to_string() });
                        }
                    }
                }
                WorkerReq::FetchDetail { owner, repo, number } => {
                    match github::fetch_issue_detail(&owner, &repo, number) {
                        Ok(detail) => {
                            let _ = tx.send(WorkerMsg::DetailReady { number, detail });
                        }
                        Err(err) => {
                            let _ = tx.send(WorkerMsg::Error { message: err.to_string() });
                        }
                    }
                }
            }
        }
    })
}

fn main() -> Result<()> {
    let (owner, repo_name) = match repo::current_repo() {
        Ok(repo) => repo,
        Err(err) => {
            eprintln!("Repo error: {err}");
            return Ok(());
        }
    };

    let owner_name = owner.clone();
    let repo_name_copy = repo_name.clone();

    let (tx_req, rx_req) = mpsc::channel();
    let (tx_msg, rx_msg) = mpsc::channel();
    let _worker = spawn_worker(tx_msg, rx_req);

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
    if !app.issues.is_empty() {
        app.list_loading = true;
        let _ = tx_req.send(WorkerReq::FetchList {
            owner: owner_name.clone(),
            repo: repo_name_copy.clone(),
        });
    }

    let result = run_app(&mut terminal, &mut app, &owner, &repo_name, rx_msg, tx_req);

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
    rx: mpsc::Receiver<WorkerMsg>,
    tx: mpsc::Sender<WorkerReq>,
) -> io::Result<()> {
    loop {
        handle_background(app, owner, repo_name, &tx);
        drain_worker(app, &rx);
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

fn ensure_detail(app: &mut App, owner: &str, repo_name: &str, tx: &mpsc::Sender<WorkerReq>) {
    let Some(number) = app.selected_issue_number() else { return; };
    if app.pending_details.contains(&number) {
        return;
    }
    if app.detail_cache.contains_key(&number) {
        return;
    }
    if let Ok(detail) = cache::read_detail(number) {
        app.detail_cache.insert(number, detail);
        // 背景更新は続けるが、表示は即キャッシュを使う
    }
    if !app.pending_details.contains(&number) {
        app.pending_details.insert(number);
        let _ = tx.send(WorkerReq::FetchDetail {
            owner: owner.to_string(),
            repo: repo_name.to_string(),
            number,
        });
    }
}

fn handle_background(app: &mut App, owner: &str, repo_name: &str, tx: &mpsc::Sender<WorkerReq>) {
    app.tick_spinner();
    if app.reload_requested {
        app.reload_requested = false;
        app.list_loading = true;
        let _ = tx.send(WorkerReq::FetchList {
            owner: owner.to_string(),
            repo: repo_name.to_string(),
        });
    }
    if let Some(number) = app.selected_issue_number() {
        app.last_selected = Some(number);
        ensure_detail(app, owner, repo_name, tx);
    }
}

fn drain_worker(app: &mut App, rx: &mpsc::Receiver<WorkerMsg>) {
    while let Ok(msg) = rx.try_recv() {
        match msg {
            WorkerMsg::ListReady { issues } => {
                app.list_loading = false;
                let prev_number = app.selected_issue_number();
                app.issues = issues.clone();
                if let Some(num) = prev_number {
                    if let Some(idx) = app.issues.iter().position(|i| i.number == num) {
                        app.selected = idx;
                    } else {
                        app.selected = 0;
                    }
                } else {
                    app.selected = 0;
                }
                app.list_scroll = 0;
                app.preview_scroll = 0;
                app.preview_hscroll = 0;
                app.pending_details.clear();
                app.error = None;
                let _ = cache::write_issues(&issues);
            }
            WorkerMsg::DetailReady { number, detail } => {
                app.pending_details.remove(&number);
                app.detail_cache.insert(number, detail.clone());
                let _ = cache::write_detail(number, &detail);
            }
            WorkerMsg::Error { message } => {
                app.list_loading = false;
                app.set_error(message);
            }
        }
    }
}
