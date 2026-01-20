use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthChar;

use crate::app::App;

fn apply_hscroll(text: &str, offset: usize, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    let mut skipped = 0;
    let mut out_width = 0;
    let mut out = String::new();
    for ch in text.chars() {
        let w = UnicodeWidthChar::width(ch).unwrap_or(0);
        if skipped + w <= offset {
            skipped += w;
            continue;
        }
        if out_width + w > max_width {
            break;
        }
        out.push(ch);
        out_width += w;
        if out_width == max_width {
            break;
        }
    }
    out
}

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let use_horizontal = area.width >= area.height * 2;
    let chunks = if use_horizontal {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area)
    };

    let list_height = chunks[0].height.saturating_sub(2) as usize;
    let selected = app.selected;
    let max_index = app.issues.len().saturating_sub(1);
    let mut list_scroll = app.list_scroll.min(max_index);
    if selected < list_scroll {
        list_scroll = selected;
    } else if selected >= list_scroll + list_height {
        list_scroll = selected.saturating_sub(list_height.saturating_sub(1));
    }
    app.list_scroll = list_scroll;
    let list_start = list_scroll;
    let list_end = (list_start + list_height).min(app.issues.len());

    let list_items: Vec<ListItem> = app
        .issues
        .iter()
        .enumerate()
        .skip(list_start)
        .take(list_end.saturating_sub(list_start))
        .map(|(idx, issue)| {
            let is_selected = idx == app.selected;
            let mut style = Style::default();
            if is_selected {
                style = style.bg(Color::DarkGray).add_modifier(Modifier::BOLD);
            }

            ListItem::new(Line::from(vec![
                Span::styled(format!("#{}", issue.number), Style::default().fg(Color::Yellow)),
                Span::raw(" "),
                Span::styled(
                    apply_hscroll(issue.title.as_str(), app.list_hscroll, chunks[0].width.saturating_sub(8) as usize),
                    style,
                ),
            ]))
        })
        .collect();

    let list_focus = if app.focus == crate::app::Focus::List { "*" } else { "" };
    let list_spin = if app.list_loading { app.spinner_char().to_string() } else { String::new() };
    let list_title = if list_spin.is_empty() {
        format!("Open {}", list_focus).trim_end().to_string()
    } else {
        format!("Open {} {}", list_focus, list_spin).trim_end().to_string()
    };
    let list = List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title(list_title));
    frame.render_widget(list, chunks[0]);

    let preview_focus = if app.focus == crate::app::Focus::Preview { "*" } else { "" };
    let preview_loading = app.selected_issue_number().map(|n| app.pending_details.contains(&n)).unwrap_or(false);
    let preview_spin = if preview_loading { app.spinner_char().to_string() } else { String::new() };
    let preview_title = if preview_spin.is_empty() {
        format!("Preview {}", preview_focus).trim_end().to_string()
    } else {
        format!("Preview {} {}", preview_focus, preview_spin).trim_end().to_string()
    };
    let right_block = Block::default().borders(Borders::ALL).title(preview_title);
    let raw_lines = if let Some(err) = app.error.as_deref() {
        vec![err.to_string()]
    } else if let Some(detail) = app
        .selected_issue_number()
        .and_then(|num| app.detail_cache.get(&num))
    {
        render_detail(detail)
    } else if app.issues.is_empty() {
        vec!["No open issues".to_string()]
    } else {
        vec![String::new()]
    };

    let visible_width = chunks[1].width.saturating_sub(2) as usize;
    let right_lines = if app.wrap_enabled {
        wrap_lines(&raw_lines, visible_width)
    } else {
        raw_lines
            .into_iter()
            .map(|line| apply_hscroll(&line, app.preview_hscroll, visible_width))
            .collect()
    };

    let visible_height = chunks[1].height.saturating_sub(2) as usize;
    app.preview_max_lines = right_lines.len();
    app.preview_page_size = visible_height;
    let start = app.preview_scroll.min(right_lines.len());
    let end = (start + visible_height).min(right_lines.len());
    let paragraph = Paragraph::new(right_lines[start..end].join("\n")).block(right_block);
    frame.render_widget(paragraph, chunks[1]);
}

fn render_detail(detail: &crate::github::IssueDetail) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!("#{} {}", detail.number, detail.title));
    lines.push(format!("Author: {}", detail.user.login));
    lines.push(format!("Created: {}", detail.created_at));
    lines.push(String::new());
    if let Some(body) = detail.body.as_ref().map(|b| b.trim()).filter(|b| !b.is_empty()) {
        lines.push(body.to_string());
    } else {
        lines.push("(no body)".to_string());
    }
    lines.push(String::new());
    lines.push(format!("Comments: {}", detail.comments.len()));
    for comment in &detail.comments {
        lines.push(String::new());
        lines.push(format!("{} @ {}", comment.user.login, comment.created_at));
        lines.push(comment.body.clone());
    }
    lines
}

fn wrap_lines(lines: &[String], width: usize) -> Vec<String> {
    if width == 0 {
        return Vec::new();
    }
    let mut out = Vec::new();
    for line in lines {
        if line.is_empty() {
            out.push(String::new());
            continue;
        }
        let mut current = String::new();
        let mut current_width = 0;
        for ch in line.chars() {
            let w = UnicodeWidthChar::width(ch).unwrap_or(0);
            if current_width + w > width && !current.is_empty() {
                out.push(current);
                current = String::new();
                current_width = 0;
            }
            current.push(ch);
            current_width += w;
            if current_width >= width {
                out.push(current);
                current = String::new();
                current_width = 0;
            }
        }
        if !current.is_empty() {
            out.push(current);
        }
    }
    out
}
