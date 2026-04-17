use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::config::defaults::{parse_default_config, generate_default_config_yaml};
use crate::config::manager::{AppConfig, RawConfig};
use crate::input::history::HistoryManager;
use crate::input::macros::MacroManager;
use crate::input::templates::TemplateManager;
use crate::output::highlight::highlight_line;
use crate::output::search::SearchState;
use crate::widgets::display::DisplayWidget;
use crate::widgets::input::InputBuffer;

// ─── Application mode ────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Normal,
    HistorySearch,
    OutputSearch,
    TemplateMenu,
    TemplateForm,
}

/// State for the inline parameter form panel.
struct TemplateForm {
    template_idx: usize,
    inputs: Vec<InputBuffer>,
    active_input: usize,
    show_dropdown: bool,
    dropdown_idx: usize,
}

impl TemplateForm {
    fn new(template_idx: usize, template: &crate::input::templates::Template) -> Self {
        let inputs = template
            .params
            .iter()
            .map(|p| {
                let mut buf = InputBuffer::new();
                if let Some(ref default) = p.default {
                    for ch in default.chars() {
                        buf.insert(ch);
                    }
                }
                buf
            })
            .collect();
        Self {
            template_idx,
            inputs,
            active_input: 0,
            show_dropdown: false,
            dropdown_idx: 0,
        }
    }

    fn active_input(&self) -> &InputBuffer {
        &self.inputs[self.active_input]
    }

    fn active_input_mut(&mut self) -> &mut InputBuffer {
        &mut self.inputs[self.active_input]
    }

    fn submit(&self, template: &crate::input::templates::Template) -> String {
        let values: Vec<_> = self
            .inputs
            .iter()
            .enumerate()
            .map(|(i, buf)| (template.params[i].name.clone(), buf.text().to_string()))
            .collect();
        template.expand(&values)
    }
}

// ─── Main application ────────────────────────────────────────────────

pub struct App {
    mode: Mode,
    target_program: Option<String>,
    child_stdin: Option<std::process::ChildStdin>,
    child_alive: bool,
    running: bool,

    display: DisplayWidget,
    input: InputBuffer,
    config: AppConfig,

    history: HistoryManager,
    macros: MacroManager,
    templates: TemplateManager,
    search: SearchState,

    // Mode-specific state
    history_query: String,
    history_search_idx: usize,
    template_idx: usize,
    form: Option<TemplateForm>,

    // Channel from child stdout reader thread
    output_rx: crossbeam_channel::Receiver<String>,
}

impl App {
    // ─── Construction ────────────────────────────────────────────────

    pub fn new(target_program: Option<String>) -> Result<Self> {
        let config = Self::load_config();
        let history_path = if config.history_persist {
            Some(Self::config_dir().join("history.json"))
        } else {
            None
        };

        let mut history = HistoryManager::new(config.history_max_size, history_path);
        let _ = history.load();

        let macros = MacroManager::new(config.macros.clone());
        let templates = TemplateManager::new(config.templates.clone());

        let (_tx, rx) = crossbeam_channel::unbounded();
        // Placeholder channel; start_child() replaces it with a real one.

        Ok(Self {
            mode: Mode::Normal,
            target_program,
            child_stdin: None,
            child_alive: false,
            running: true,
            display: DisplayWidget::new(config.timestamp_format.clone(), config.timestamp_enabled),
            input: InputBuffer::new(),
            config,
            history,
            macros,
            templates,
            search: SearchState::new(),
            history_query: String::new(),
            history_search_idx: 0,
            template_idx: 0,
            form: None,
            output_rx: rx,
        })
    }

    // ─── Config helpers ──────────────────────────────────────────────

    fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default())
            .join("cli-terminal")
    }

    fn config_path() -> PathBuf {
        Self::config_dir().join("config.yaml")
    }

    fn load_config() -> AppConfig {
        let path = Self::config_path();

        // Ensure config directory exists.
        if let Err(e) = fs::create_dir_all(path.parent().unwrap()) {
            eprintln!("Warning: cannot create config dir: {e}");
            return Self::fallback_config();
        }

        if !path.exists() {
            let yaml = generate_default_config_yaml();
            let _ = fs::write(&path, &yaml);
            let raw: RawConfig =
                serde_yaml::from_str(&yaml).unwrap_or(Self::fallback_raw());
            return AppConfig::from_raw(raw);
        }

        let raw: RawConfig = match fs::read_to_string(&path) {
            Ok(content) => match serde_yaml::from_str(&content) {
                Ok(cfg) => cfg,
                Err(e) => {
                    eprintln!("Warning: invalid config YAML: {e}, using defaults");
                    return Self::fallback_config();
                }
            },
            Err(e) => {
                eprintln!("Warning: cannot read config: {e}, using defaults");
                return Self::fallback_config();
            }
        };

        AppConfig::from_raw(raw)
    }

    fn fallback_config() -> AppConfig {
        AppConfig::from_raw(parse_default_config())
    }

    fn fallback_raw() -> RawConfig {
        parse_default_config()
    }

    fn reload_config(&mut self) {
        let config = Self::load_config();
        self.config = config.clone();
        self.macros.update(config.macros);
        self.templates.update(config.templates);
        // Update display timestamp settings.
        self.display = DisplayWidget::new(config.timestamp_format, config.timestamp_enabled);
        self.display.add_message("Config reloaded");
    }

    // ─── Main entry ──────────────────────────────────────────────────

    pub fn run(&mut self) -> Result<()> {
        self.start_child()?;

        enable_raw_mode()?;
        let mut stdout = io::stderr();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.event_loop(&mut terminal);

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        let _ = terminal.show_cursor();

        self.cleanup();
        result
    }

    // ─── Child process ───────────────────────────────────────────────

    fn start_child(&mut self) -> Result<()> {
        let Some(ref program) = self.target_program else {
            return Ok(());
        };

        let mut child = Command::new(program)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        eprintln!("Connected to: {program}");
        self.child_alive = true;

        let stdin = child.stdin.take();

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        // We need to send both stdout and stderr through the same channel.
        // Create the channel here and store the receiver.
        let (tx, rx) = crossbeam_channel::unbounded();
        self.output_rx = rx;

        // Merge stderr into stdout channel.
        let tx_stderr = tx.clone();
        thread::spawn(move || {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                match line {
                    Ok(l) => {
                        let _ = tx_stderr.send(format!("[stderr] {l}"));
                    }
                    Err(_) => break,
                }
            }
        });

        // Main stdout reader. Send exit notification when child exits.
        thread::spawn(move || {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(l) => {
                        let _ = tx.send(l);
                    }
                    Err(_) => break,
                }
            }
            // Notify that the child has exited.
            let _ = tx.send("__CLI_TERMINAL_CHILD_EXITED__".to_string());
        });

        self.child_stdin = stdin;
        Ok(())
    }

    fn send_command(&mut self, command: &str) {
        if command.trim().is_empty() {
            return;
        }

        self.history.add(command);
        self.display.add_command(format!("> {command}"));

        if let Some(ref mut stdin) = self.child_stdin {
            let _ = stdin.write_all(command.as_bytes());
            let _ = stdin.write_all(b"\n");
            let _ = stdin.flush();
        } else {
            // No child: print to stdout.
            println!("{command}");
        }
    }

    fn drain_child_output(&mut self) {
        while let Ok(line) = self.output_rx.try_recv() {
            if line == "__CLI_TERMINAL_CHILD_EXITED__" {
                self.child_alive = false;
                self.display.add_message("Child process exited");
                break;
            }
            self.display.add_line(line);
        }
    }

    fn cleanup(&mut self) {
        let _ = self.history.save();
        if self.child_alive
            && let Some(ref mut child_stdin) = self.child_stdin
        {
            let _ = child_stdin.write_all(b"exit\n");
            let _ = child_stdin.flush();
        }
    }

    // ─── Event loop ──────────────────────────────────────────────────

    fn event_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stderr>>) -> Result<()> {
        loop {
            self.drain_child_output();
            terminal.draw(|frame| self.render(frame))?;

            if !self.running {
                break;
            }

            if event::poll(std::time::Duration::from_millis(50))?
                && let Event::Key(key) = event::read()?
            {
                if key.kind == event::KeyEventKind::Release {
                    continue;
                }
                match self.mode {
                    Mode::Normal => self.handle_normal(key),
                    Mode::HistorySearch => self.handle_history_search(key),
                    Mode::OutputSearch => self.handle_output_search(key),
                    Mode::TemplateMenu => self.handle_template_menu(key),
                    Mode::TemplateForm => self.handle_template_form(key),
                }
            }
        }
        Ok(())
    }

    // ─── Normal mode key handling ────────────────────────────────────

    fn handle_normal(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => self.exit(),
            (KeyModifiers::CONTROL, KeyCode::Char('r')) => self.enter_history_search(),
            (KeyModifiers::CONTROL, KeyCode::Char('l')) => self.reload_config(),
            (KeyModifiers::CONTROL, KeyCode::Char('e')) => self.export_output(),
            (_, KeyCode::Char('/')) => self.enter_output_search(),
            (_, KeyCode::F(1)) => self.enter_template_menu(),
            (_, KeyCode::Enter) => self.submit_command(),
            (_, KeyCode::Backspace) => self.input.backspace(),
            (_, KeyCode::Delete) => self.input.delete(),
            (_, KeyCode::Left) => self.input.move_left(),
            (_, KeyCode::Right) => self.input.move_right(),
            (_, KeyCode::Home) => self.input.move_home(),
            (_, KeyCode::End) => self.input.move_end(),
            (_, KeyCode::Tab) => self.try_autocomplete(),
            (_, KeyCode::Esc) => {
                self.search.clear();
            }
            (_, KeyCode::Char('C')) => self.toggle_collapse(),
            (_, KeyCode::Char(c)) => self.input.insert(c),
            _ => {}
        }

        // Check macro keys (F2-F12) — these override default F-key behavior.
        if let KeyCode::F(n @ 2..=12) = key.code {
            let key_name = format!("F{n}");
            if let Some(cmd) = self.macros.get(&key_name) {
                let cmd = cmd.to_string();
                self.send_command(&cmd);
            }
        }
    }

    // ─── History search mode ─────────────────────────────────────────

    fn handle_history_search(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Enter) => {
                let matches = self.history.search(&self.history_query);
                if self.history_search_idx < matches.len() {
                    let selected = matches[self.history_search_idx].to_string();
                    self.input.clear();
                    for ch in selected.chars() {
                        self.input.insert(ch);
                    }
                }
                self.mode = Mode::Normal;
            }
            (_, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                self.mode = Mode::Normal;
            }
            (_, KeyCode::Up) if self.history_search_idx > 0 => {
                self.history_search_idx -= 1;
            }
            (_, KeyCode::Down) => {
                let count = self.history.search(&self.history_query).len();
                if self.history_search_idx + 1 < count {
                    self.history_search_idx += 1;
                }
            }
            (_, KeyCode::Backspace) => {
                self.history_query.pop();
                self.history_search_idx = 0;
            }
            (_, KeyCode::Char(c)) => {
                self.history_query.push(c);
                self.history_search_idx = 0;
            }
            _ => {}
        }
    }

    // ─── Output search mode ──────────────────────────────────────────

    fn handle_output_search(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Enter) => {
                self.search.set_query(&self.history_query);
                self.search.execute(&self.display.raw_lines());
                self.mode = Mode::Normal;
            }
            (_, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                self.search.clear();
                self.mode = Mode::Normal;
            }
            (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char('n')) => {
                self.search.next_match();
            }
            (_, KeyCode::Char('N')) => {
                self.search.prev_match();
            }
            (_, KeyCode::Backspace) => {
                self.history_query.pop();
                self.search.set_query(&self.history_query);
                self.search.execute(&self.display.raw_lines());
            }
            (_, KeyCode::Char(c)) => {
                self.history_query.push(c);
                self.search.set_query(&self.history_query);
                self.search.execute(&self.display.raw_lines());
            }
            _ => {}
        }
    }

    // ─── Template menu mode ──────────────────────────────────────────

    fn handle_template_menu(&mut self, key: KeyEvent) {
        let templates = self.templates.templates();
        match (key.modifiers, key.code) {
            (_, KeyCode::Enter) => {
                if self.template_idx < templates.len() {
                    self.enter_template_form(self.template_idx);
                }
                // Don't switch to Normal — form mode handles submit/cancel.
            }
            (_, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                self.mode = Mode::Normal;
            }
            (_, KeyCode::Up) if self.template_idx > 0 => {
                self.template_idx -= 1;
            }
            (_, KeyCode::Down) if self.template_idx + 1 < templates.len() => {
                self.template_idx += 1;
            }
            _ => {}
        }
    }

    // ─── Action handlers ─────────────────────────────────────────────

    fn exit(&mut self) {
        self.running = false;
    }

    fn enter_history_search(&mut self) {
        self.mode = Mode::HistorySearch;
        self.history_query = String::new();
        self.history_search_idx = 0;
    }

    fn enter_output_search(&mut self) {
        self.mode = Mode::OutputSearch;
        self.history_query = String::new();
        self.search.set_query("");
    }

    fn enter_template_menu(&mut self) {
        if self.templates.templates().is_empty() {
            self.display.add_message("No templates configured");
            return;
        }
        self.mode = Mode::TemplateMenu;
        self.template_idx = 0;
    }

    fn submit_command(&mut self) {
        let command = self.input.text().to_string();
        self.send_command(&command);
        self.input.clear();
    }

    fn try_autocomplete(&mut self) {
        let text = self.input.text();
        if text.is_empty() {
            return;
        }

        // First try command completions from config.
        let completions: Vec<_> = self
            .config
            .commands
            .iter()
            .filter(|(label, _)| label.starts_with(text))
            .map(|(label, _)| label.clone())
            .collect();

        if completions.len() == 1 {
            // Exact match — replace input.
            self.input.clear();
            for ch in completions[0].chars() {
                self.input.insert(ch);
            }
        } else if completions.len() > 1 {
            // Ambiguous — show options.
            let match_list = completions.join(", ");
            self.display.add_message(&format!("Matches: {match_list}"));
        }
    }

    fn export_output(&mut self) {
        match self.display.export() {
            Ok(path) => {
                self.display
                    .add_message(&format!("Output exported to: {}", path.display()));
            }
            Err(e) => {
                self.display.add_message(&format!("Export failed: {e}"));
            }
        }
    }

    fn toggle_collapse(&mut self) {
        match self.display.collapse_manager().toggle_last() {
            Some(true) => self.display.add_message("Block collapsed"),
            Some(false) => self.display.add_message("Block expanded"),
            None => {} // No regions to toggle.
        }
    }

    fn enter_template_form(&mut self, template_idx: usize) {
        let templates = self.templates.templates();
        let template = templates.get(template_idx).unwrap();
        self.form = Some(TemplateForm::new(template_idx, template));
        self.mode = Mode::TemplateForm;
    }

    fn handle_template_form(&mut self, key: KeyEvent) {
        let form = match self.form.as_mut() {
            Some(f) => f,
            None => {
                self.mode = Mode::Normal;
                return;
            }
        };

        let templates = self.templates.templates();
        let template = templates.get(form.template_idx).unwrap();
        let param = template.params.get(form.active_input);

        match (key.modifiers, key.code) {
            (_, KeyCode::Enter) => {
                if form.show_dropdown {
                    if let Some(p) = param {
                        if !p.options.is_empty() {
                            let val = p.options[form.dropdown_idx].clone();
                            let buf = form.active_input_mut();
                            buf.clear();
                            for ch in val.chars() {
                                buf.insert(ch);
                            }
                            form.show_dropdown = false;
                        }
                    }
                } else {
                    let command = form.submit(template);
                    self.send_command(&command);
                    self.form = None;
                    self.mode = Mode::Normal;
                }
            }
            (_, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                self.form = None;
                self.mode = Mode::TemplateMenu;
            }
            (_, KeyCode::Char('d')) if key.modifiers == KeyModifiers::CONTROL => {
                if let Some(p) = param {
                    if !p.options.is_empty() {
                        form.show_dropdown = !form.show_dropdown;
                        form.dropdown_idx = 0;
                    }
                }
            }
            (_, KeyCode::Tab) => {
                if form.show_dropdown {
                    let opt_count = template
                        .params
                        .get(form.active_input)
                        .map(|p| p.options.len())
                        .unwrap_or(0);
                    if form.dropdown_idx + 1 < opt_count {
                        form.dropdown_idx += 1;
                    }
                } else {
                    form.active_input = (form.active_input + 1) % form.inputs.len();
                }
            }
            (_, KeyCode::BackTab) => {
                if form.active_input > 0 {
                    form.active_input -= 1;
                } else {
                    form.active_input = form.inputs.len().saturating_sub(1);
                }
            }
            (_, KeyCode::Backspace) => form.active_input_mut().backspace(),
            (_, KeyCode::Delete) => form.active_input_mut().delete(),
            (_, KeyCode::Left) => form.active_input_mut().move_left(),
            (_, KeyCode::Right) => form.active_input_mut().move_right(),
            (_, KeyCode::Home) => form.active_input_mut().move_home(),
            (_, KeyCode::End) => form.active_input_mut().move_end(),
            (_, KeyCode::Char(c)) => form.active_input_mut().insert(c),
            _ => {}
        }
    }

    // ─── Rendering ───────────────────────────────────────────────────

    fn render(&mut self, frame: &mut ratatui::Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(1),
                Constraint::Length(3),
            ])
            .split(frame.area());

        self.render_output(frame, chunks[0]);
        self.render_status(frame, chunks[1]);
        self.render_input(frame, chunks[2]);

        // Overlay mode-specific UI.
        if self.mode == Mode::TemplateMenu {
            self.render_template_menu(frame);
        }
    }

    fn render_template_menu(&self, frame: &mut ratatui::Frame) {
        let templates = self.templates.templates();
        let area = frame.area();

        // Center a popup box.
        let width = 50.min(area.width.saturating_sub(4));
        let height = (templates.len() as u16 + 4).min(area.height.saturating_sub(2));
        let popup = ratatui::layout::Rect::new(
            (area.width - width) / 2,
            (area.height - height) / 2,
            width,
            height,
        );

        let items: Vec<Line> = templates
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let style = if i == self.template_idx {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                let marker = if i == self.template_idx { "► " } else { "  " };
                Line::from(Span::styled(format!("{}{}", marker, t.label), style))
            })
            .collect();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Templates (Enter to select, Esc to cancel) ");

        let list = Paragraph::new(items).block(block);

        // Dim the background.
        let background = Paragraph::new("").style(Style::default().bg(Color::DarkGray));
        frame.render_widget(background, area);
        frame.render_widget(list, popup);
    }

    fn render_output(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let rules = &self.config.highlights;

        // Build styled lines from display widget.
        let raw = self.display.raw_lines();
        let mut styled_lines: Vec<Line> = Vec::with_capacity(raw.len());

        for (i, line) in raw.iter().enumerate() {
            // Check collapse state.
            let collapse = self.display.collapse_manager();
            if collapse.is_hidden(i) {
                continue;
            }
            if let Some(header) = collapse.collapsed_header_at(i) {
                styled_lines.push(Line::from(Span::styled(
                    format!("[+ {header}]"),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
                continue;
            }

            // Apply highlight rules.
            let mut highlighted = highlight_line(line, rules, Some(Color::Yellow));

            // Apply search match highlighting.
            if self.search.active() {
                if self.search.is_current_match(i) {
                    // Current match: bright background.
                    for span in &mut highlighted.spans {
                        span.style = span.style.bg(Color::Magenta);
                    }
                } else if self.search.is_match(i) {
                    // Other match: subtle background.
                    for span in &mut highlighted.spans {
                        if span.style.bg.is_none() {
                            span.style = span.style.bg(Color::DarkGray);
                        }
                    }
                }
            }

            styled_lines.push(highlighted);
        }

        let para = Paragraph::new(styled_lines)
            .wrap(Wrap { trim: false })
            .scroll((
                self.display.len().saturating_sub(area.height as usize) as u16,
                0,
            ));

        frame.render_widget(para, area);
    }

    fn render_status(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let mut spans = Vec::new();

        // Mode indicator.
        let mode_text = match self.mode {
            Mode::Normal => "NORMAL",
            Mode::HistorySearch => "HISTORY SEARCH",
            Mode::OutputSearch => "OUTPUT SEARCH",
            Mode::TemplateMenu => "TEMPLATE MENU",
            Mode::TemplateForm => "TEMPLATE FORM",
        };
        spans.push(Span::styled(
            format!(" {mode_text} "),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ));

        // Search query.
        if self.mode == Mode::OutputSearch {
            spans.push(Span::styled(
                format!(" /{} ", self.history_query),
                Style::default().fg(Color::Yellow),
            ));
            if self.search.match_count() > 0 {
                spans.push(Span::raw(format!(
                    "  ({} matches)",
                    self.search.match_count()
                )));
            }
        }

        if self.mode == Mode::HistorySearch {
            spans.push(Span::styled(
                format!(" ?{} ", self.history_query),
                Style::default().fg(Color::Cyan),
            ));
            let count = self.history.search(&self.history_query).len();
            spans.push(Span::raw(format!("  ({count} matches)")));
        }

        // Target program.
        if let Some(ref target) = self.target_program {
            spans.push(Span::raw(format!(" | {target}")));
        }

        frame.render_widget(Paragraph::new(Line::from(spans)), area);
    }

    fn render_input(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let input_block = Block::default()
            .borders(Borders::TOP)
            .title(" Command ")
            .border_style(Style::default().fg(Color::DarkGray));

        let prompt = "$ ";
        let cursor_byte = self.input.cursor_byte();
        let text = self.input.text();

        let before_cursor = &text[..cursor_byte];
        let after_cursor = &text[cursor_byte..];

        let mut spans = vec![
            Span::styled(prompt, Style::default().fg(Color::Green)),
            Span::raw(before_cursor),
        ];
        if !after_cursor.is_empty() {
            // Show cursor as inverted space.
            spans.push(Span::styled(
                after_cursor.chars().next().unwrap_or(' ').to_string(),
                Style::default()
                    .bg(Color::White)
                    .fg(Color::Black)
                    .add_modifier(Modifier::REVERSED),
            ));
            let rest = &after_cursor[after_cursor
                .chars()
                .next()
                .map(|c| c.len_utf8())
                .unwrap_or(0)..];
            if !rest.is_empty() {
                spans.push(Span::raw(rest));
            }
        } else {
            spans.push(Span::styled(
                " ",
                Style::default()
                    .bg(Color::White)
                    .fg(Color::Black)
                    .add_modifier(Modifier::REVERSED),
            ));
        }

        let para = Paragraph::new(Line::from(spans)).block(input_block);
        frame.render_widget(para, area);
    }
}
