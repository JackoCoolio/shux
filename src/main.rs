#![allow(unused_imports)]

use std::{
    error::Error,
    fs::File,
    io::{self, Read},
    os::{
        fd::{AsRawFd, OwnedFd},
        unix::process::CommandExt,
    },
    process::Command,
    sync::mpsc::{self, Receiver},
};

use crossbeam::queue::ArrayQueue;
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind,
};
use handler::{HandleEvent, HandleEventResult};
use nix::{
    errno::Errno,
    pty::{openpty, OpenptyResult},
};
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Clone, Copy)]
struct Dimensions {
    rows: u16,
    columns: u16,
}

struct PtyOutput {
    bytes: [u8; 255],
    size: u8,
}

struct Job {
    command: String,
    dimensions: Option<Dimensions>,
    stdout: Receiver<PtyOutput>,
    child_id: u32,
    output: Vec<u8>,
}

impl Job {
    pub fn set_dimensions(&mut self, dimensions: Dimensions) {
        if self
            .dimensions
            .is_some_and(|current_dimensions| current_dimensions == dimensions)
        {
            // same dimensions as previous; no need to notify pty
            return;
        }

        self.dimensions = Some(dimensions);
    }

    #[expect(dead_code)]
    pub fn reset_dimensions(&mut self) {
        self.dimensions = None;
    }
}

struct App {
    prompt: Prompt,
    jobs: Vec<Job>,
}

impl App {
    fn draw(&mut self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Length(3)])
            .split(frame.area());

        let jobs_area = layout[0];

        let jobs_areas = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints(self.jobs.iter().map(|_job| Constraint::Fill(1)))
            .split(jobs_area);
        for (i, (job, job_area)) in self.jobs.iter_mut().zip(jobs_areas.iter()).enumerate() {
            let block = Block::bordered()
                .border_type(ratatui::widgets::BorderType::Rounded)
                .title(
                    Title::from(format!("JOB {i}: '{}'", job.command.as_str().italic()))
                        .alignment(ratatui::layout::Alignment::Left),
                );

            let inner_area = block.inner(*job_area);

            let block = {
                let dimensions = Dimensions {
                    rows: inner_area.height,
                    columns: inner_area.width,
                };

                job.set_dimensions(dimensions);

                block.title(
                    Title::from(
                        format!("{}x{}", dimensions.rows, dimensions.columns)
                            .italic()
                            .dim(),
                    )
                    .alignment(ratatui::layout::Alignment::Right),
                )
            };

            frame.render_widget(block, *job_area);
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

    fn start_job(&mut self, command: String) -> Result<&Job, Errno> {
        let OpenptyResult { slave, master } = openpty(None, None)?;

        let slave = slave.as_raw_fd();

        let mut child = unsafe {
            Command::new(command.clone())
                .pre_exec(move || {
                    if nix::libc::login_tty(slave) != 0 {
                        return Err(io::Error::last_os_error());
                    }
                    close_fds();

                    Ok(())
                })
                .spawn()
                .unwrap_or_else(|err| panic!("unable to spawn command: error {err}"))
        };

        let child_id = child.id();

        std::thread::spawn(move || {
            child.wait().unwrap();
            let _ = nix::unistd::close(slave);
        });

        let (sender, receiver) = mpsc::channel();

        std::thread::spawn(move || {
            let mut file = File::from(master);

            loop {
                let mut buf = [0; 255];

                let n = match file.read(&mut buf) {
                    Err(err) => {
                        continue;
                        if let Some(raw_os_err) = err.raw_os_error() {
                            let raw_os_err = Errno::from_raw(raw_os_err);
                            panic!("fatal error on read pty: {raw_os_err}");
                        } else {
                            panic!("other fatal error on read pty: {err}");
                        }
                    }
                    Ok(n) => n,
                };

                sender
                    .send(PtyOutput {
                        bytes: buf,
                        size: n as u8,
                    })
                    .unwrap();
            }
        });

        self.jobs.push(Job {
            command,
            dimensions: None,
            child_id,
            stdout: receiver,
        });

        Ok(self.jobs.last().unwrap())
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
                    self.start_job(command);

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

unsafe fn close_fds() {
    if cfg!(any(target_os = "macos", target_os = "ios")) {
        close_fds::set_fds_cloexec(3, &[])
    } else {
        close_fds::close_open_fds(3, &[])
    }
}
