use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::github::{Issue, IssueDetail};

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum Focus {
    #[default]
    List,
    Preview,
}

#[derive(Debug)]
pub struct App {
    pub should_quit: bool,
    pub issues: Vec<Issue>,
    pub error: Option<String>,
    pub selected: usize,
    pub detail_cache: HashMap<u64, IssueDetail>,
    pub list_scroll: usize,
    pub preview_scroll: usize,
    pub preview_max_lines: usize,
    pub preview_page_size: usize,
    pub focus: Focus,
    pub wrap_enabled: bool,
    pub list_hscroll: usize,
    pub preview_hscroll: usize,
    pub reload_requested: bool,
    pub list_loading: bool,
    pub pending_details: HashSet<u64>,
    pub spinner_index: usize,
    pub last_spinner_tick: Instant,
    pub last_selected: Option<u64>,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            issues: Vec::new(),
            error: None,
            selected: 0,
            detail_cache: HashMap::new(),
            list_scroll: 0,
            preview_scroll: 0,
            preview_max_lines: 0,
            preview_page_size: 0,
            focus: Focus::List,
            wrap_enabled: true,
            list_hscroll: 0,
            preview_hscroll: 0,
            reload_requested: false,
            list_loading: false,
            pending_details: HashSet::new(),
            spinner_index: 0,
            last_spinner_tick: Instant::now(),
            last_selected: None,
        }
    }

    pub fn set_error(&mut self, err: impl Into<String>) {
        self.error = Some(err.into());
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.error = None;
            self.preview_scroll = 0;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.issues.len() {
            self.selected += 1;
            self.error = None;
            self.preview_scroll = 0;
        }
    }

    pub fn selected_issue_number(&self) -> Option<u64> {
        self.issues.get(self.selected).map(|issue| issue.number)
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::List => Focus::Preview,
            Focus::Preview => Focus::List,
        };
    }

    pub fn spinner_char(&self) -> char {
        let frames = ['-', '\\', '|', '/'];
        frames[self.spinner_index % frames.len()]
    }

    pub fn tick_spinner(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_spinner_tick).as_millis() >= 120 {
            self.spinner_index = (self.spinner_index + 1) % 4;
            self.last_spinner_tick = now;
        }
    }

    pub fn toggle_wrap(&mut self) {
        self.wrap_enabled = !self.wrap_enabled;
        if self.wrap_enabled {
            self.preview_hscroll = 0;
        }
    }

    pub fn preview_scroll_left(&mut self) {
        if self.preview_hscroll > 0 {
            self.preview_hscroll -= 1;
        }
    }

    pub fn preview_scroll_right(&mut self) {
        self.preview_hscroll = self.preview_hscroll.saturating_add(1);
    }

    pub fn list_scroll_left(&mut self) {
        if self.list_hscroll > 0 {
            self.list_hscroll -= 1;
        }
    }

    pub fn list_scroll_right(&mut self) {
        self.list_hscroll = self.list_hscroll.saturating_add(1);
    }

    pub fn preview_scroll_up(&mut self) {
        if self.preview_scroll > 0 {
            self.preview_scroll -= 1;
        }
    }

    pub fn preview_scroll_down(&mut self) {
        if self.preview_scroll + 1 < self.preview_max_lines {
            self.preview_scroll += 1;
        }
    }

    pub fn preview_page_up(&mut self) {
        let page = self.preview_page_size.max(1);
        self.preview_scroll = self.preview_scroll.saturating_sub(page);
    }

    pub fn preview_page_down(&mut self) {
        let page = self.preview_page_size.max(1);
        let max_scroll = self.preview_max_lines.saturating_sub(1);
        self.preview_scroll = (self.preview_scroll + page).min(max_scroll);
    }
}
