use bon::{bon, builder, Builder};
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Position, Rect},
    text::{Line, Span},
    widgets::Widget,
};

use crate::handler::{HandleEvent, HandleEventResult};

#[derive(Builder)]
pub struct TextArea {
    #[builder(default = vec!["".to_string()])]
    lines: Vec<String>,
    max_lines: usize,

    prefix: Option<String>,

    #[builder(default = 0)]
    cursor_row: usize,
    #[builder(default = 0)]
    cursor_col: usize,

    #[builder(default = 0)]
    scroll_offset: usize,
}

impl TextArea {
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

    fn prefix_len(&self) -> usize {
        match &self.prefix {
            Some(prefix) => prefix.len(),
            None => 0,
        }
    }

    pub fn get_rendered_cursor_position(&self, rect: &Rect) -> Position {
        let mut pos = rect.as_position();
        pos.x += self.cursor_col as u16;
        pos.y += (self.cursor_row - self.scroll_offset) as u16;

        let prefix_len = self.prefix_len();

        if prefix_len > 0 && self.cursor_row == 0 {
            pos.x += prefix_len as u16;
        }

        pos
    }
}

impl HandleEvent for TextArea {
    fn handle_event(&mut self, event: Event) -> HandleEventResult {
        match event {
            Event::Key(key_event) => {
                if key_event.kind == KeyEventKind::Release {
                    return HandleEventResult::Bubbled(event);
                }

                match key_event.code {
                    KeyCode::Char(char) => {
                        if self.insert_char(char) {
                            HandleEventResult::Handled
                        } else {
                            // didn't work - bubble up
                            HandleEventResult::Bubbled(event)
                        }
                    }
                    KeyCode::Left => {
                        self.cursor_col = self.cursor_col.saturating_sub(1);
                        HandleEventResult::Handled
                    }
                    KeyCode::Right => {
                        let cur_line = self.get_current_line();
                        self.cursor_col = self.cursor_col.saturating_add(1).min(cur_line.len());
                        HandleEventResult::Handled
                    }
                    KeyCode::Up | KeyCode::Down => todo!("switching lines"),
                    KeyCode::Backspace => {
                        self.backspace_char();
                        HandleEventResult::Handled
                    }
                    KeyCode::Enter => {
                        let (left, right) = self.get_current_line().split_at(self.cursor_col);

                        let mut new_buf = Vec::new();
                        for i in 0..self.cursor_row {
                            new_buf.push(self.lines[i].clone());
                        }

                        new_buf.push(left.to_string());
                        new_buf.push(right.to_string());

                        for i in self.cursor_row + 1..self.lines.len() {
                            new_buf.push(self.lines[i].clone());
                        }

                        self.cursor_row += 1;
                        self.cursor_col = 0;

                        self.lines = new_buf;

                        HandleEventResult::Handled
                    }
                    _ => HandleEventResult::Bubbled(event),
                }
            }
            _ => HandleEventResult::Bubbled(event),
        }
    }
}

impl Widget for &TextArea {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let mut row_lines = area.rows().zip(self.lines.iter().skip(self.scroll_offset));

        if let Some(prefix) = &self.prefix {
            let Some((row, line)) = row_lines.next() else {
                return;
            };

            Line::from(Span::from(format!("{}{}", prefix, line))).render(row, buf);
        }

        for (row, line) in row_lines {
            Line::from(Span::from(line)).render(row, buf);
        }
    }
}
