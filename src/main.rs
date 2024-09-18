#![allow(unused_imports)]

use std::{error::Error, io};

use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind,
};
use handler::{HandleEvent, HandleEventResult};
use prompt::{Prompt, PromptEvent};
use ratatui::{
    layout::{Constraint, Layout, Position, Rect},
    style::Stylize,
    text::{Line, Span, Text},
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
        prompt: Prompt::new(TextArea::builder().prefix("$ ".into()).max_lines(3).build()),
        jobs: Vec::new(),
    };

    app.run(terminal)?;

    ratatui::restore();

    Ok(())
}

struct Job {
    command: String,
}

struct App {
    prompt: Prompt,
    jobs: Vec<Job>,
}

impl App {
    fn draw(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Length(3)])
            .split(frame.area());

        let jobs_area = layout[0];

        let jobs_areas = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints(self.jobs.iter().map(|_job| Constraint::Fill(1)))
            .split(jobs_area);
        for (i, (Job { command }, job_area)) in self.jobs.iter().zip(jobs_areas.iter()).enumerate()
        {
            let block = Block::bordered()
                .border_type(ratatui::widgets::BorderType::Rounded)
                .title(Title::from(format!(
                    "JOB {i}: '{}'",
                    command.as_str().italic()
                )))
                .title_alignment(ratatui::layout::Alignment::Left);

            let inner_area = block.inner(*job_area);
            frame.render_widget(block, *job_area);

            let line = Paragraph::new(Text::from(Span::from(command)));
            frame.render_widget(line, inner_area);
        }

        let input_area = layout[1];

        let title = Title::from(" ~/example/status@hostname ".bold());

        let block = Block::bordered()
            .border_type(ratatui::widgets::BorderType::Rounded)
            .title(title)
            .title_alignment(ratatui::layout::Alignment::Left);

        let inner_area = block.inner(input_area);
        frame.render_widget(block, input_area);
        frame.render_widget(&self.prompt, inner_area);

        let cursor_pos = self.prompt.get_rendered_cursor_position(&inner_area);
        frame.set_cursor_position(cursor_pos);
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        loop {
            // draw the current state
            terminal.draw(|frame| self.draw(frame))?;

            // wait for an event
            let event = event::read()?;

            let HandleEventResult::Bubbled(event) = self.prompt.handle_event(event) else {
                // prompt handled it
                continue;
            };

            let event = match event {
                PromptEvent::Enter(command) => {
                    self.jobs.push(Job { command });

                    self.prompt.clear();

                    continue;
                }
                PromptEvent::Bubble(event) => event,
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
