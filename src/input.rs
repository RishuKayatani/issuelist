use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{App, Focus};

pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('r') | KeyCode::F(5) => app.reload_requested = true,
        KeyCode::Enter => app.toggle_focus(),
        KeyCode::Up => {
            if app.focus == Focus::List {
                app.move_up();
            } else {
                app.preview_scroll_up();
            }
        }
        KeyCode::Char('k') => {
            if app.focus == Focus::List {
                app.move_up();
            } else {
                app.preview_scroll_up();
            }
        }
        KeyCode::Down => {
            if app.focus == Focus::List {
                app.move_down();
            } else {
                app.preview_scroll_down();
            }
        }
        KeyCode::Char('j') => {
            if app.focus == Focus::List {
                app.move_down();
            } else {
                app.preview_scroll_down();
            }
        }
        KeyCode::PageUp | KeyCode::Char('b') => app.preview_page_up(),
        KeyCode::PageDown | KeyCode::Char('f') => app.preview_page_down(),
        KeyCode::Char('g') => app.preview_scroll = 0,
        KeyCode::Char('w') => app.toggle_wrap(),
        KeyCode::Char('G') => app.preview_scroll = app.preview_max_lines.saturating_sub(1),
        KeyCode::Left | KeyCode::Char('h') => {
            if app.focus == Focus::List {
                app.list_scroll_left();
            } else {
                app.preview_scroll_left();
            }
        }
        KeyCode::Right | KeyCode::Char('l') => {
            if app.focus == Focus::List {
                app.list_scroll_right();
            } else {
                app.preview_scroll_right();
            }
        }
        _ => {}
    }
}
