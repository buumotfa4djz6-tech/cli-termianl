# F1 Template Form Enhancement Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the terminal-suspend parameter prompt with an inline TUI form panel, add parameter options support in config, and improve the template list display.

**Architecture:** Add a new `Mode::TemplateForm` that renders an overlay panel inside the TUI. Each parameter gets an `InputBuffer` for editing, with optional dropdown for parameters that have `options` defined in config. The template list popup is widened to show command previews.

**Tech Stack:** Rust, ratatui 0.29, crossterm 0.28, serde

---

## File Structure

| File | Responsibility |
|------|---------------|
| `src/config/manager.rs:51-56` | Add `options` and `default` fields to `TemplateParam` |
| `src/input/templates.rs` | Carry `options` field through `TemplateParam` |
| `src/app.rs` | Add `Mode::TemplateForm`, `TemplateForm` state, form rendering, form key handling, improve template list rendering |
| `src/config/defaults.yaml` | Add `options` to a few representative params as examples |

---

### Task 1: Add `options` and `default` to `TemplateParam`

**Files:**
- Modify: `src/config/manager.rs:51-56`

- [ ] **Step 1: Add new fields to `TemplateParam`**

In `src/config/manager.rs`, update the `TemplateParam` struct (lines 51-56) to support optional parameter options and defaults:

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TemplateParam {
    pub name: String,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub default: Option<String>,
}
```

The `#[serde(default)]` ensures existing configs without these fields still deserialize correctly.

- [ ] **Step 2: Build to verify**

Run: `cargo check`
Expected: Compiles without errors.

- [ ] **Step 3: Commit**

```bash
git add src/config/manager.rs
git commit -m "feat: add options and default fields to TemplateParam"
```

---

### Task 2: Carry `options` through `Template` struct

**Files:**
- Modify: `src/input/templates.rs:1-74`

- [ ] **Step 1: Add `options` field to `TemplateParam` in templates module**

The `TemplateParam` in `templates.rs` (line 3) is imported from `crate::config::manager`. It already has `options` now. But the `Template` struct only stores `params: Vec<TemplateParam>` — the `options` field comes through automatically since it's part of `TemplateParam`.

Verify that `TemplateParam` is re-exported from config/manager and used directly. No code change needed in `templates.rs` — the `options` field flows through `Template::from_config()` since it copies `cfg.params` directly into `self.params`.

- [ ] **Step 2: Verify with build**

Run: `cargo check`
Expected: Compiles without errors.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "chore: verify options flow through Template struct"
```

---

### Task 3: Add `TemplateForm` state and `Mode::TemplateForm`

**Files:**
- Modify: `src/app.rs:34-40` (Mode enum)
- Modify: `src/app.rs:44-67` (App struct)
- Modify: `src/input/templates.rs` (add `options()` method to `TemplateParam` and `Template`)

- [ ] **Step 1: Add `TemplateForm` to `Mode` enum**

In `src/app.rs`, add a new variant to the `Mode` enum (around line 39):

```rust
#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Normal,
    HistorySearch,
    OutputSearch,
    TemplateMenu,
    TemplateForm,
}
```

- [ ] **Step 2: Add helper method to `Template` for options lookup**

In `src/input/templates.rs`, add methods to `Template` and `TemplateParam`:

```rust
// In templates.rs, inside impl Template:

/// Get the TemplateParam by index, with its options.
pub fn param_at(&self, idx: usize) -> Option<&TemplateParam> {
    self.params.get(idx)
}
```

- [ ] **Step 3: Add `TemplateForm` state struct**

In `src/app.rs`, add a new struct after the `App` struct definition:

```rust
/// State for the inline parameter form panel.
struct TemplateForm {
    template_idx: usize,       // index into self.templates
    inputs: Vec<InputBuffer>,  // one per parameter
    active_input: usize,       // which parameter is being edited
    show_dropdown: bool,       // whether dropdown is open for select-type params
    dropdown_idx: usize,       // selected option index in dropdown
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
```

- [ ] **Step 4: Add `form` field to `App` struct**

In `src/app.rs`, add a new field to the `App` struct (around line 63):

```rust
    // Template form state
    form: Option<TemplateForm>,
```

Initialize it in `App::new()` (around line 104):

```rust
            form: None,
```

- [ ] **Step 5: Build to verify**

Run: `cargo check`
Expected: Compiles without errors.

- [ ] **Step 6: Commit**

```bash
git add src/app.rs src/input/templates.rs
git commit -m "feat: add TemplateForm state struct and Mode::TemplateForm"
```

---

### Task 4: Enter form mode from template menu

**Files:**
- Modify: `src/app.rs:432-459` (handle_template_menu)
- Modify: `src/app.rs:542-583` (select_template_with_params and interactive_template_prompt)

- [ ] **Step 1: Replace `select_template_with_params` to enter form mode**

Replace the `select_template_with_params` method (lines 542-556) and `interactive_template_prompt` method (lines 558-583). The old code suspended the terminal; the new code just enters form mode:

```rust
    fn enter_template_form(&mut self, template_idx: usize) {
        let template = self.templates.templates().get(template_idx).cloned();
        if let Some(template) = template {
            // We need access to the template reference, not an owned clone.
            // Store the index and build inputs from the template params.
            self.form = Some(TemplateForm::new(template_idx, self.templates.templates().get(template_idx).unwrap()));
            self.mode = Mode::TemplateForm;
        }
    }
```

Wait — `Template` doesn't impl `Clone`. Let me use the index-only approach. Update `TemplateForm::new` to take a reference to the template:

```rust
    fn enter_template_form(&mut self, template_idx: usize) {
        let templates = self.templates.templates();
        let template = templates.get(template_idx).unwrap();
        self.form = Some(TemplateForm::new(template_idx, template));
        self.mode = Mode::TemplateForm;
    }
```

- [ ] **Step 2: Update `handle_template_menu` Enter handler**

In `src/app.rs`, in the `handle_template_menu` method (line 432), replace the Enter branch (lines 435-446):

```rust
            (_, KeyCode::Enter) => {
                if self.template_idx < templates.len() {
                    self.enter_template_form(self.template_idx);
                }
                // Don't switch to Normal mode here — form mode handles that on submit/cancel.
            }
```

- [ ] **Step 3: Build to verify**

Run: `cargo check`
Expected: Compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add src/app.rs
git commit -m "feat: enter form mode instead of suspending terminal"
```

---

### Task 5: Form key handling

**Files:**
- Modify: `src/app.rs:310-317` (event_loop dispatch)
- Modify: `src/app.rs` (add new `handle_template_form` method)

- [ ] **Step 1: Add form dispatch in event_loop**

In `src/app.rs`, the event_loop dispatch (around line 311-316) needs a new arm:

```rust
                    Mode::TemplateForm => self.handle_template_form(key),
```

- [ ] **Step 2: Implement `handle_template_form` method**

Add a new method to `App`:

```rust
    fn handle_template_form(&mut self, key: KeyEvent) {
        let form = match self.form.as_mut() {
            Some(f) => f,
            None => { self.mode = Mode::Normal; return; }
        };

        let templates = self.templates.templates();
        let template = templates.get(form.template_idx).unwrap();
        let param = template.params.get(form.active_input);

        match (key.modifiers, key.code) {
            // Submit form
            (_, KeyCode::Enter) => {
                if form.show_dropdown {
                    // Select from dropdown
                    if let Some(p) = param {
                        if p.options.len() > 0 {
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
                    // Submit: expand and send command
                    let command = form.submit(template);
                    self.send_command(&command);
                    self.form = None;
                    self.mode = Mode::Normal;
                }
            }
            // Cancel: return to template list
            (_, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                self.form = None;
                self.mode = Mode::TemplateMenu;
            }
            // Toggle dropdown for params with options
            (_, KeyCode::Char('d')) if key.modifiers == KeyModifiers::CONTROL => {
                if let Some(p) = param {
                    if !p.options.is_empty() {
                        form.show_dropdown = !form.show_dropdown;
                        form.dropdown_idx = 0;
                    }
                }
            }
            // Navigate between input fields
            (_, KeyCode::Tab) => {
                if form.show_dropdown {
                    // Navigate dropdown
                    if form.dropdown_idx + 1 < template.params.get(form.active_input).map(|p| p.options.len()).unwrap_or(0) {
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
            // Normal text editing in active input
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
```

- [ ] **Step 3: Build to verify**

Run: `cargo check`
Expected: Compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add src/app.rs
git commit -m "feat: add key handling for template form mode"
```

---

### Task 6: Render the form panel

**Files:**
- Modify: `src/app.rs:587-606` (render method)
- Modify: `src/app.rs` (add new `render_template_form` method)
- Modify: `src/app.rs:708-755` (render_status - add TemplateForm mode text)

- [ ] **Step 1: Add form rendering dispatch**

In `src/app.rs`, in the `render` method (around line 602-605), add form overlay rendering:

```rust
        // Overlay mode-specific UI.
        if self.mode == Mode::TemplateMenu {
            self.render_template_menu(frame);
        }
        if self.mode == Mode::TemplateForm {
            self.render_template_form(frame);
        }
```

- [ ] **Step 2: Implement `render_template_form` method**

Add a new rendering method after `render_template_menu`:

```rust
    fn render_template_form(&self, frame: &mut ratatui::Frame) {
        let form = match self.form.as_ref() {
            Some(f) => f,
            None => return,
        };
        let templates = self.templates.templates();
        let template = templates.get(form.template_idx).unwrap();
        let area = frame.area();

        // Calculate panel dimensions.
        let max_label_width: usize = template
            .params
            .iter()
            .map(|p| p.prompt.as_deref().unwrap_or(&p.name).chars().count())
            .max()
            .unwrap_or(10);

        let panel_lines: usize = 3  // title + preview + spacing
            + template.params.len() * 2  // each param: label + input row
            + if form.show_dropdown {
                template.params.get(form.active_input).map(|p| p.options.len()).unwrap_or(0) + 2
            } else {
                0
            }
            + 2;  // help line + border padding

        let width = (max_label_width + 30).min(70) as u16;
        let height = (panel_lines as u16).min(area.height.saturating_sub(2));

        let popup_width = width.min(area.width.saturating_sub(4));
        let popup_height = height.min(area.height.saturating_sub(2));
        let popup = ratatui::layout::Rect::new(
            (area.width - popup_width) / 2,
            (area.height - popup_height) / 2,
            popup_width,
            popup_height,
        );

        let mut content: Vec<Line> = Vec::new();

        // Title: template name.
        content.push(Line::from(Span::styled(
            template.label.clone(),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));
        content.push(Line::from(""));

        // Command preview: show command with filled values.
        let mut preview = template.command.clone();
        for (i, p) in template.params.iter().enumerate() {
            if let Some(f) = form.inputs.get(i) {
                if !f.is_empty() {
                    preview = preview.replace(&format!("{{{}}}", p.name), f.text());
                }
            }
        }
        content.push(Line::from(Span::styled(
            format!("  {preview}"),
            Style::default().fg(Color::DarkGray),
        )));
        content.push(Line::from(""));

        // Parameter input fields.
        for (i, param) in template.params.iter().enumerate() {
            let label = param.prompt.as_deref().unwrap_or(&param.name);
            let is_active = i == form.active_input;

            let label_style = if is_active {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            // Build the input text representation.
            let buf = &form.inputs[i];
            let text = buf.text();
            let cursor_byte = buf.cursor_byte();
            let before = &text[..cursor_byte.min(text.len())];
            let after = &text[cursor_byte.min(text.len())..];

            let indicator = if !param.options.is_empty() { " ▼" } else { "" };

            let input_style = if is_active {
                Style::default().fg(Color::White).bg(Color::DarkGray)
            } else {
                Style::default().fg(Color::Gray)
            };

            let cursor_style = if is_active {
                Style::default().bg(Color::White).fg(Color::Black)
            } else {
                input_style
            };

            let mut spans = vec![
                Span::styled(format!("  {:>width$} ", label, width = max_label_width), label_style),
                Span::styled("[", input_style),
                Span::raw(before.to_string()),
            ];
            if is_active {
                spans.push(Span::styled(" ", cursor_style));
            }
            if !after.is_empty() {
                spans.push(Span::styled(after.to_string(), input_style));
            }
            spans.push(Span::styled(format!("]{indicator}"), input_style));
            content.push(Line::from(spans));

            // Dropdown for params with options.
            if is_active && form.show_dropdown && !param.options.is_empty() {
                for (j, opt) in param.options.iter().enumerate() {
                    let opt_style = if j == form.dropdown_idx {
                        Style::default().fg(Color::Black).bg(Color::Cyan)
                    } else {
                        Style::default().fg(Color::Gray)
                    };
                    let marker = if j == form.dropdown_idx { "► " } else { "   " };
                    content.push(Line::from(Span::styled(
                        format!("    {}{}", marker, opt),
                        opt_style,
                    )));
                }
            }

            content.push(Line::from(""));
        }

        // Help text.
        let help = if form.show_dropdown {
            "Tab=select  Enter=pick  Esc=close dropdown"
        } else {
            "Tab=next  Shift+Tab=prev  Enter=submit  Esc=cancel  Ctrl+D=options"
        };
        content.push(Line::from(Span::styled(
            format!("  {help}"),
            Style::default().fg(Color::DarkGray),
        )));

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Parameters ");

        let paragraph = Paragraph::new(content).block(block);

        // Dim background.
        let background = Paragraph::new("").style(Style::default().bg(Color::DarkGray));
        frame.render_widget(background, area);
        frame.render_widget(paragraph, popup);
    }
```

- [ ] **Step 3: Update render_status for TemplateForm mode**

In `render_status` (line 712), add the mode text:

```rust
            Mode::TemplateForm => "TEMPLATE FORM",
```

- [ ] **Step 4: Build to verify**

Run: `cargo check`
Expected: Compiles without errors.

- [ ] **Step 5: Commit**

```bash
git add src/app.rs
git commit -m "feat: render inline template form panel with inputs and dropdown"
```

---

### Task 7: Improve template list display

**Files:**
- Modify: `src/app.rs:608-650` (render_template_menu)

- [ ] **Step 1: Update template list to show command preview**

Replace the `render_template_menu` method (lines 608-650) to show the command alongside the label and auto-size the popup:

```rust
    fn render_template_menu(&self, frame: &mut ratatui::Frame) {
        let templates = self.templates.templates();
        let area = frame.area();

        // Calculate width based on longest command preview.
        let max_len = templates
            .iter()
            .map(|t| {
                // "► " + label + " | " + first 20 chars of command
                let cmd_preview: String = t
                    .command
                    .chars()
                    .take(30)
                    .collect();
                t.label.chars().count() + cmd_preview.chars().count() + 5
            })
            .max()
            .unwrap_or(30);

        let width = (max_len as u16 + 4).min(80).min(area.width.saturating_sub(4));
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
                let cmd_preview: String = t
                    .command
                    .chars()
                    .take(30)
                    .collect();
                let is_selected = i == self.template_idx;
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                let cmd_style = if is_selected {
                    Style::default().fg(Color::DarkGray).bg(Color::Cyan)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                let marker = if is_selected { "► " } else { "  " };
                Line::from(vec![
                    Span::styled(format!("{}{}", marker, t.label), style),
                    Span::raw(" "),
                    Span::styled(format!("| {cmd_preview}"), cmd_style),
                ])
            })
            .collect();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Templates (Enter=select, Esc=cancel) ");

        let list = Paragraph::new(items).block(block);

        // Dim the background.
        let background = Paragraph::new("").style(Style::default().bg(Color::DarkGray));
        frame.render_widget(background, area);
        frame.render_widget(list, popup);
    }
```

- [ ] **Step 2: Build and test**

Run: `cargo build`
Expected: Compiles without errors.

- [ ] **Step 3: Commit**

```bash
git add src/app.rs
git commit -m "feat: show command preview in template list popup"
```

---

### Task 8: Add example options to defaults.yaml

**Files:**
- Modify: `src/config/defaults.yaml`

- [ ] **Step 1: Add `options` to representative parameters**

In `src/config/defaults.yaml`, update a few key templates with `options`:

```yaml
  - label: "mscontrol"
    command: "mscontrol {side} {angle_rate} {deep_rate}"
    params:
      - name: side
        prompt: "Side"
        options: [left, right]
        default: "left"
      - name: angle_rate
        prompt: "Angle rate"
        options: [0.5, 1.0, 1.5, 2.0]
        default: "1.0"
      - name: deep_rate
        prompt: "Depth rate"
        options: [0.5, 1.0, 1.5, 2.0]
        default: "1.0"

  - label: "update_rcm"
    command: "update_rcm"
    params: []

  - label: "get_rcm"
    command: "get_rcm"
    params: []
```

And for `inject`:

```yaml
  - label: "inject"
    command: "inject {mode} {d_inj}"
    params:
      - name: mode
        prompt: "Injection mode"
        options: [step, continuous]
        default: "step"
      - name: d_inj
        prompt: "Injection volume"
```

And for `autoseq`:

```yaml
  - label: "autoseq"
    command: "autoseq {file}"
    params:
      - name: file
        prompt: "Sequence file path"
```

(Keep all other templates unchanged — they'll use free-text input since they have no `options`.)

- [ ] **Step 2: Build and verify config loads**

Run: `cargo build && cargo run`
Expected: App starts, F1 shows templates with options. Select a template with options, verify dropdown appears with Ctrl+D.

- [ ] **Step 3: Commit**

```bash
git add src/config/defaults.yaml
git commit -m "feat: add example options to key template parameters"
```

---

## Self-Review

### 1. Spec coverage

| Requirement | Task |
|------------|------|
| Replace suspend-terminal prompt with inline TUI form | Task 4, 5, 6 |
| Add `options` field to config params | Task 1, 2 |
| Add `default` field to config params | Task 1, 3 |
| Form with Tab navigation between fields | Task 5 |
| Dropdown for params with options | Task 5, 6 |
| Command preview in form | Task 6 |
| Command preview in template list | Task 7 |
| Example options in defaults | Task 8 |

All requirements covered.

### 2. Placeholder scan

No TBD/TODO/fill-in-later patterns found. All code steps contain actual implementation code.

### 3. Type consistency

- `TemplateParam.options: Vec<String>` — defined in Task 1, used consistently in Tasks 5, 6, 8
- `TemplateParam.default: Option<String>` — defined in Task 1, used in Task 3 (`TemplateForm::new`)
- `TemplateForm` struct fields match usage in Tasks 5 and 6
- `Mode::TemplateForm` variant matches dispatch in Tasks 4 and 5

### 4. Scope check

Focused scope — only 4 files modified, one feature set. Appropriate for a single implementation plan.

---

Plan complete. Saved to `docs/superpowers/plans/2026-04-17-template-form-enhancement.md`.
