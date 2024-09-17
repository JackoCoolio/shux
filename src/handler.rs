use crossterm::event::Event;

pub enum HandleEventResult<E> {
    Handled,
    Bubbled(E),
}

impl<E> From<Option<E>> for HandleEventResult<E> {
    fn from(value: Option<E>) -> Self {
        match value {
            Some(value) => Self::Bubbled(value),
            None => Self::Handled,
        }
    }
}

impl<E> From<HandleEventResult<E>> for Option<E> {
    fn from(value: HandleEventResult<E>) -> Self {
        match value {
            HandleEventResult::Bubbled(value) => Some(value),
            HandleEventResult::Handled => None,
        }
    }
}

pub trait HandleEvent {
    type Event;
    fn handle_event(&mut self, event: Event) -> HandleEventResult<Self::Event>;
}
