#![allow(unused_imports)]

use std::{error::Error, io};

use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind,
};
use ratatui::{
    layout::{Constraint, Layout, Position, Rect},
    style::Stylize,
    text::{Line, Span},
    widgets::{block::Title, Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();

    terminal.clear()?;

    let mut app = App {
        text_area: "foobar".into(),
    };

    app.run(terminal)?;

    ratatui::restore();

    Ok(())
}

struct TextArea {
    lines: Vec<String>,
    max_lines: usize,

    prefix: String,

    cursor_row: usize,
    cursor_col: usize,

    scroll_offset: usize,
}

impl TextArea {
    pub fn new(mut lines: Vec<String>) -> Self {
        if lines.is_empty() {
            lines = vec!["".to_string()];
        }

        Self {
            lines,
            max_lines: 1,

            prefix: "$ ".into(),

            cursor_row: 0,
            cursor_col: 0,

            scroll_offset: 0,
        }
    }

    fn get_current_line(&self) -> &str {
        &self.lines[self.cursor_row]
    }

    fn insert_char(&mut self, char: char) -> bool {
        let Some(line) = self.lines.get_mut(self.cursor_row) else {
            return false;
        };

        // note: <=, not < here - insert works when idx == len
        assert!(self.cursor_col <= line.len());

        line.insert(self.cursor_col, char);

        self.cursor_col += 1;

        true
    }

    /// Backspaces at the current cursor position.
    fn backspace_char(&mut self) {
        if self.cursor_col == 0 {
            if self.cursor_row == 0 {
                // already at start of string
                return;
            } else {
                todo!("removing lines");
            }
        }

        let new_col = self.cursor_col.saturating_sub(1);
        self.lines[self.cursor_row].remove(new_col);
        self.cursor_col = new_col;
    }

    pub fn handle_event(&mut self, event: &Event) -> bool {
        match event {
            Event::Key(key_event) => {
                if key_event.kind == KeyEventKind::Release {
                    return false;
                }

                match key_event.code {
                    KeyCode::Char(char) => {
                        if self.insert_char(char) {
                            true
                        } else {
                            // didn't work - bubble up
                            false
                        }
                    }
                    KeyCode::Left => {
                        self.cursor_col = self.cursor_col.saturating_sub(1);
                        true
                    }
                    KeyCode::Right => {
                        let cur_line = self.get_current_line();
                        self.cursor_col = self.cursor_col.saturating_add(1).min(cur_line.len());
                        true
                    }
                    KeyCode::Up | KeyCode::Down => todo!("switching lines"),
                    KeyCode::Backspace => {
                        self.backspace_char();
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn get_cursor_position(&self, rect: &Rect) -> Position {
        let mut pos = rect.as_position();
        pos.x += self.cursor_col as u16;
        pos.y += (self.cursor_row - self.scroll_offset) as u16;

        let prefix_len = self.prefix.len();
        if prefix_len > 0 && self.cursor_row == 0 {
            pos.x += prefix_len as u16;
        }

        pos
    }
}

impl From<&str> for TextArea {
    fn from(value: &str) -> Self {
        Self::new(value.lines().map(String::from).collect())
    }
}

impl Widget for &TextArea {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let mut row_lines = area.rows().zip(self.lines.iter().skip(self.scroll_offset));

        if !self.prefix.is_empty() {
            let Some((row, line)) = row_lines.next() else {
                return;
            };

            Line::from(Span::from(format!("{}{}", self.prefix, line))).render(row, buf);
        }

        for (row, line) in row_lines {
            Line::from(Span::from(line)).render(row, buf);
        }
    }
}

struct App {
    text_area: TextArea,
}

impl App {
    fn draw(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Length(3)])
            .split(frame.area());

        let input_area = layout[1];

        let title = Title::from(" ~/example/status@hostname ".bold());

        let block = Block::bordered()
            .border_type(ratatui::widgets::BorderType::Rounded)
            .title(title)
            .title_alignment(ratatui::layout::Alignment::Left);

        let inner_area = block.inner(input_area);
        frame.render_widget(block, input_area);
        frame.render_widget(&self.text_area, inner_area);

        let cursor_pos = self.text_area.get_cursor_position(&inner_area);
        frame.set_cursor_position(cursor_pos);
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            let event = event::read()?;
            self.text_area.handle_event(&event);

            if let event::Event::Key(key) = event {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Esc {
                    return Ok(());
                }
            }
        }
    }
}
