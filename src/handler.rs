use crossterm::event::Event;

pub enum HandleEventResult {
    Handled,
    Bubbled(Event),
}

impl From<Option<Event>> for HandleEventResult {
    fn from(value: Option<Event>) -> Self {
        match value {
            Some(value) => Self::Bubbled(value),
            None => Self::Handled,
        }
    }
}

impl From<HandleEventResult> for Option<Event> {
    fn from(value: HandleEventResult) -> Self {
        match value {
            HandleEventResult::Bubbled(value) => Some(value),
            HandleEventResult::Handled => None,
        }
    }
}

pub trait HandleEvent {
    fn handle_event(&mut self, event: Event) -> HandleEventResult;
}
