use std::ops::{Deref, DerefMut};

use bon::{bon, builder};
use crossterm::event::{Event, KeyEventKind};
use ratatui::widgets::Widget;

use crate::{
    handler::{HandleEvent, HandleEventResult},
    text_area::{TextArea, TextAreaBuilder},
};

pub struct Prompt {
    text_area: TextArea,
}

impl From<TextArea> for Prompt {
    fn from(value: TextArea) -> Self {
        Self::new(value)
    }
}

pub enum PromptEvent {
    /// The user pressed the enter key in the prompt.
    Enter(String),
    Bubble(<TextArea as HandleEvent>::Event),
}

impl Prompt {
    pub fn new(text_area: TextArea) -> Self {
        Self { text_area }
    }
}

impl HandleEvent for Prompt {
    type Event = PromptEvent;

    fn handle_event(&mut self, event: Event) -> crate::handler::HandleEventResult<Self::Event> {
        match event {
            Event::Key(key_event) if key_event.kind != KeyEventKind::Release => {
                if key_event.code == crossterm::event::KeyCode::Enter {
                    return HandleEventResult::Bubbled(PromptEvent::Enter(
                        self.text_area.current_line().to_owned(),
                    ));
                }
            }
            _ => (),
        }

        match self.text_area.handle_event(event) {
            HandleEventResult::Handled => HandleEventResult::Handled,
            HandleEventResult::Bubbled(b) => HandleEventResult::Bubbled(PromptEvent::Bubble(b)),
        }
    }
}

impl Deref for Prompt {
    type Target = TextArea;

    fn deref(&self) -> &Self::Target {
        &self.text_area
    }
}

impl DerefMut for Prompt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.text_area
    }
}

impl Widget for &Prompt {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        self.text_area.render(area, buf)
    }
}
