use std::ops::Deref;

use bon::{bon, builder};
use crossterm::event::Event;
use ratatui::widgets::Widget;

use crate::{
    handler::HandleEvent,
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

enum PromptEvent {
    Enter(String),
}

impl Prompt {
    pub fn new(text_area: TextArea) -> Self {
        Self { text_area }
    }
}

impl HandleEvent for Prompt {
    fn handle_event(&mut self, event: Event) -> crate::handler::HandleEventResult {
        self.text_area.handle_event(event)
    }
}

impl Deref for Prompt {
    type Target = TextArea;

    fn deref(&self) -> &Self::Target {
        &self.text_area
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
