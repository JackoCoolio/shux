#![allow(unused_imports)]

use std::{error::Error, io};

use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind,
};
use handler::{HandleEvent, HandleEventResult};
use prompt::Prompt;
use ratatui::{
    layout::{Constraint, Layout, Position, Rect},
    style::Stylize,
    text::{Line, Span},
    widgets::{block::Title, Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};
use text_area::TextArea;

pub mod handler;
pub mod prompt;
pub mod text_area;

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();

    terminal.clear()?;

    let mut app = App {
        text_area: TextArea::builder().prefix("$ ".into()).max_lines(3).build(),
    };

    app.run(terminal)?;

    ratatui::restore();

    Ok(())
}

struct App {
    text_area: TextArea,
}

impl App {
    fn draw(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Length(5)])
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

        let cursor_pos = self.text_area.get_rendered_cursor_position(&inner_area);
        frame.set_cursor_position(cursor_pos);
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        loop {
            // draw the current state
            terminal.draw(|frame| self.draw(frame))?;

            // wait for an event
            let event = event::read()?;

            let HandleEventResult::Bubbled(event) = self.text_area.handle_event(event) else {
                // prompt handled it
                continue;
            };

            if let event::Event::Key(key) = event {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Esc {
                    return Ok(());
                }

                if key.kind == KeyEventKind::Press && key.code == KeyCode::Enter {}
            }
        }
    }
}
