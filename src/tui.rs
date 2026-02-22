//! Terminal UI for the time-based task scheduler using ratatui.

use crate::job::Job;
use crate::queue::QueueManager;
use chrono::{TimeZone, Utc};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

const MAX_LOG_LINES: usize = 200;

pub struct AppState {
    pub queue: Arc<Mutex<QueueManager>>,
    pub log_rx: Receiver<String>,
    pub worker_tx: Sender<Job>,
    pub log_lines: Vec<String>,
    pub list_state: ListState,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub input_field: InputField,
    pub message: Option<(String, std::time::Instant)>,
    pub available_functions: Vec<String>,
    pub function_index: usize,
}

#[derive(Clone, Copy, Default)]
pub enum InputMode {
    #[default]
    Normal,
    AddTask,
}

#[derive(Clone, Copy)]
pub enum InputField {
    Time,
    Priority,
    Description,
    Function,
}

impl Default for InputField {
    fn default() -> Self {
        InputField::Time
    }
}

/// Temporary state for the "add task" form (one set per submission).
pub struct AddTaskForm {
    pub time: String,
    pub priority: String,
    pub description: String,
    pub function: String,
}

impl Default for AddTaskForm {
    fn default() -> Self {
        Self {
            time: String::new(),
            priority: String::new(),
            description: String::new(),
            function: String::new(),
        }
    }
}

impl AppState {
    pub fn new(
        queue: Arc<Mutex<QueueManager>>,
        log_rx: Receiver<String>,
        worker_tx: Sender<Job>,
        available_functions: Vec<String>,
    ) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            queue,
            log_rx,
            worker_tx,
            log_lines: Vec::with_capacity(MAX_LOG_LINES),
            list_state,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            input_field: InputField::Time,
            message: None,
            available_functions,
            function_index: 0,
        }
    }

    fn drain_log(&mut self) {
        while let Ok(line) = self.log_rx.try_recv() {
            self.log_lines.push(line);
            if self.log_lines.len() > MAX_LOG_LINES {
                self.log_lines.remove(0);
            }
        }
    }

    fn pending_jobs(&mut self) -> Vec<Job> {
        if let Ok(mut q) = self.queue.lock() {
            q.snapshot()
        } else {
            Vec::new()
        }
    }

    fn selected_job_id(&self, jobs: &[Job]) -> Option<Uuid> {
        let i = self.list_state.selected()?;
        jobs.get(i).map(|j| j.id)
    }

    fn remove_selected(&mut self) {
        let jobs = self.pending_jobs();
        if let Some(id) = self.selected_job_id(&jobs) {
            if let Ok(mut q) = self.queue.lock() {
                q.remove(id);
                self.message = Some(("Job removed.".to_string(), std::time::Instant::now()));
            }
            if let Some(sel) = self.list_state.selected() {
                let len = jobs.len();
                if len <= 1 {
                    self.list_state.select(None);
                } else {
                    self.list_state
                        .select(Some((sel + len - 1) % (len - 1).max(1)));
                }
            }
        } else {
            self.message = Some(("No job selected.".to_string(), std::time::Instant::now()));
        }
    }

    fn submit_add_task(&mut self, form: &AddTaskForm) -> bool {
        let time_str = form.time.trim();
        let priority_str = form.priority.trim();
        let desc = form.description.trim().to_string();
        let func = form.function.trim().to_string();

        if time_str.is_empty() || priority_str.is_empty() || desc.is_empty() || func.is_empty() {
            self.message = Some((
                "All fields required.".to_string(),
                std::time::Instant::now(),
            ));
            return false;
        }

        let time_str = time_str.trim_start_matches('+');
        let execution_time: i64 = match time_str.parse::<i64>() {
            Ok(val) => {
                // If the number is smaller than 1 billion, assume it's relative seconds from now.
                // Otherwise treat it as an explicit Unix timestamp.
                if val < 1_000_000_000 {
                    chrono::Utc::now().timestamp() + val
                } else {
                    val
                }
            }
            Err(_) => {
                self.message = Some((
                    "Invalid time. Enter seconds (e.g. 5) or Unix timestamp.".to_string(),
                    std::time::Instant::now(),
                ));
                return false;
            }
        };
        let priority: u8 = match priority_str.parse() {
            Ok(p) => p,
            Err(_) => {
                self.message = Some((
                    "Priority must be 0–255.".to_string(),
                    std::time::Instant::now(),
                ));
                return false;
            }
        };

        match Job::new(execution_time, priority, desc, func) {
            Ok(job) => {
                if let Ok(mut q) = self.queue.lock() {
                    q.push(job);
                    self.message = Some(("Job added.".to_string(), std::time::Instant::now()));
                }
                self.input_mode = InputMode::Normal;
                true
            }
            Err(e) => {
                self.message = Some((e.to_string(), std::time::Instant::now()));
                false
            }
        }
    }
}

pub fn run_tui(
    queue: Arc<Mutex<QueueManager>>,
    log_rx: Receiver<String>,
    worker_tx: Sender<Job>,
    available_functions: Vec<String>,
) -> std::io::Result<()> {
    use crossterm::terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
    };
    use ratatui::Terminal;
    use ratatui::backend::CrosstermBackend;

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = AppState::new(queue, log_rx, worker_tx, available_functions);
    let mut form = AddTaskForm::default();

    loop {
        app.drain_log();
        terminal.draw(|f| ui(f, &mut app, &form))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                // Handle Ctrl+C to quit from any mode
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    break;
                }

                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            app.input_mode = InputMode::AddTask;
                            form = AddTaskForm::default();
                            app.input_field = InputField::Time;
                            app.input_buffer = form.time.clone();
                        }
                        KeyCode::Char('d') | KeyCode::Delete => app.remove_selected(),
                        KeyCode::Up => {
                            let jobs = app.pending_jobs();
                            let len = jobs.len();
                            if len > 0 {
                                let i = app.list_state.selected().unwrap_or(0);
                                app.list_state.select(Some((i + len - 1) % len));
                            }
                        }
                        KeyCode::Down => {
                            let jobs = app.pending_jobs();
                            let len = jobs.len();
                            if len > 0 {
                                let i = app.list_state.selected().unwrap_or(0);
                                app.list_state.select(Some((i + 1) % len));
                            }
                        }
                        _ => {}
                    },
                    InputMode::AddTask => match key.code {
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                            app.message =
                                Some(("Cancelled.".to_string(), std::time::Instant::now()));
                        }
                        KeyCode::Enter => match app.input_field {
                            InputField::Time => {
                                form.time = app.input_buffer.clone();
                                app.input_field = InputField::Priority;
                                app.input_buffer = form.priority.clone();
                            }
                            InputField::Priority => {
                                form.priority = app.input_buffer.clone();
                                app.input_field = InputField::Description;
                                app.input_buffer = form.description.clone();
                            }
                            InputField::Description => {
                                form.description = app.input_buffer.clone();
                                app.input_field = InputField::Function;
                                if !app.available_functions.is_empty() {
                                    app.input_buffer =
                                        app.available_functions[app.function_index].clone();
                                } else {
                                    app.input_buffer = form.function.clone();
                                }
                            }
                            InputField::Function => {
                                form.function = app.input_buffer.clone();
                                if app.submit_add_task(&form) {
                                    app.input_buffer.clear();
                                }
                            }
                        },
                        KeyCode::Backspace => {
                            if !matches!(app.input_field, InputField::Function)
                                || app.available_functions.is_empty()
                            {
                                app.input_buffer.pop();
                            }
                        }
                        KeyCode::Char(c) => {
                            if !matches!(app.input_field, InputField::Function)
                                || app.available_functions.is_empty()
                            {
                                app.input_buffer.push(c);
                            }
                        }
                        KeyCode::Up => {
                            if matches!(app.input_field, InputField::Function)
                                && !app.available_functions.is_empty()
                            {
                                if app.function_index == 0 {
                                    app.function_index = app.available_functions.len() - 1;
                                } else {
                                    app.function_index -= 1;
                                }
                                app.input_buffer =
                                    app.available_functions[app.function_index].clone();
                            }
                        }
                        KeyCode::Down => {
                            if matches!(app.input_field, InputField::Function)
                                && !app.available_functions.is_empty()
                            {
                                app.function_index =
                                    (app.function_index + 1) % app.available_functions.len();
                                app.input_buffer =
                                    app.available_functions[app.function_index].clone();
                            }
                        }
                        _ => {}
                    },
                }
            }
        }

        if let Some((_, instant)) = &app.message {
            if instant.elapsed() > Duration::from_secs(3) {
                app.message = None;
            }
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn ui(f: &mut Frame, app: &mut AppState, _form: &AddTaskForm) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(f.area());

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    let jobs = app.pending_jobs();
    let title = " Pending tasks (↑/↓ select, Ctrl+A add, D remove, Q quit) ";
    let list_items: Vec<ListItem> = jobs
        .iter()
        .map(|j| {
            let ts = Utc
                .timestamp_opt(j.execution_time, 0)
                .single()
                .unwrap_or_else(Utc::now);
            let time_str = ts.format("%H:%M:%S %Y-%m-%d").to_string();
            ListItem::new(Line::from(vec![
                Span::raw(format!("{} │ P{} │ ", time_str, j.priority)),
                Span::styled(j.description.as_str(), Style::default().fg(Color::Cyan)),
            ]))
        })
        .collect();

    let list = List::new(list_items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut list_state = std::mem::take(&mut app.list_state);
    if !jobs.is_empty() && list_state.selected().is_none() {
        list_state.select(Some(0));
    }
    f.render_stateful_widget(list, main_chunks[0], &mut list_state);
    app.list_state = list_state;

    let log_text: Vec<Line> = app
        .log_lines
        .iter()
        .rev()
        .take(30)
        .map(|s| Line::from(s.as_str()))
        .collect();
    let log = Paragraph::new(log_text)
        .block(
            Block::default()
                .title(" Engine / Worker log ")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(log, main_chunks[1]);

    let help = match app.input_mode {
        InputMode::Normal => " Ctrl+A: Add task │ D: Delete selected │ Q/Esc/Ctrl+C: Quit ",
        InputMode::AddTask => {
            if matches!(app.input_field, InputField::Function)
                && !app.available_functions.is_empty()
            {
                " Enter: Submit │ Esc: Cancel │ ↑/↓: Select Function "
            } else {
                " Enter: Next field │ Esc: Cancel │ Time = Secs from now OR Unix sec "
            }
        }
    };
    let help_para = Paragraph::new(help)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Yellow));

    let input_area = chunks[1];
    if matches!(app.input_mode, InputMode::AddTask) {
        let field_name = match app.input_field {
            InputField::Time => " Time (Secs from now or Unix sec) ",
            InputField::Priority => " Priority (0-255) ",
            InputField::Description => " Description ",
            InputField::Function => {
                if app.available_functions.is_empty() {
                    " Function name "
                } else {
                    " Function name (↑/↓ to select) "
                }
            }
        };
        let prompt = format!("{}: {}", field_name, app.input_buffer);
        let input_block = Paragraph::new(prompt.as_str())
            .block(Block::default().title(" Add task ").borders(Borders::ALL))
            .style(Style::default().fg(Color::Green));
        f.render_widget(input_block, input_area);
    } else {
        f.render_widget(help_para, input_area);
    }

    if let Some((msg, _)) = &app.message {
        let area = centered_rect(60, 20, f.area());
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));
        let para = Paragraph::new(msg.as_str())
            .block(block)
            .wrap(Wrap { trim: true });
        f.render_widget(Clear, area);
        f.render_widget(para, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    let vertical = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1]);
    vertical[1]
}
