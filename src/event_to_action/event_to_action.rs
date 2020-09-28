use crate::stream_interface::events::ChatEvent;
use crate::actions::action::ActionCategory;

pub trait EventToAction {
    fn execute(&mut self, event: ChatEvent) -> Option<ActionCategory>;
    fn custom_categories(&mut self) -> Vec<String>;
}
