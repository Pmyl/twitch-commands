use crate::event_to_action::event_to_action::{EventToAction};
use crate::stream_interface::events::{ChatEvent};
use crate::utils::run_on_stream::StreamItemReceiver;
use crate::system_input::system_input::{SystemInput};
use crate::system_input::enigo::enigo_system_input::EnigoSystemInput;
use crate::actions::action::{Action, ActionCategory};

pub struct TestEventToAction {
    controller: EnigoSystemInput
}

// impl TestEventToAction {
//     pub fn new(sender: Sender<Action>) -> TestEventToAction {
//         TestEventToAction { controller: EnigoSystemInput::new(), sender }
//     }
// }

impl EventToAction for TestEventToAction {
    fn execute(&mut self, event: ChatEvent) -> Option<ActionCategory> {
        execute(event, &mut self.controller)
    }

    fn custom_categories(&mut self) -> Vec<String> {
        vec![]
    }
}

impl StreamItemReceiver for TestEventToAction {
    type Item = ChatEvent;
    type Output = Option<ActionCategory>;
    fn receive(&mut self, event: ChatEvent) -> Option<ActionCategory> {
        let maybe_action = self.execute(event);
        match maybe_action {
            Some(action) => {
                // println!("test_event_to_action::send_in_channel::({:?})", action);
                // match self.sender.send(action.clone()).await {
                //     Ok(_) => println!("test_event_to_action::send_ok"),
                //     Err(e) => println!("test_event_to_action::send_error::{}", e)
                // };
                Some(action)
            },
            _ => None
        }
    }
}

fn execute(_event: ChatEvent, _system_input: &mut impl SystemInput) -> Option<ActionCategory> {
    println!("test_event_to_action::map_to::(test action key raw 1)");
    Some(ActionCategory::Uncategorized(Action::KeyRawDown(1)))
}