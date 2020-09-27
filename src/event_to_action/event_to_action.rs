use crate::stream_interface::events::ChatEvent;
use crate::actions::action::Action;

pub trait EventToAction {
    fn execute(&mut self, event: ChatEvent) -> Option<Action>;
}
