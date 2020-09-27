use async_trait::async_trait;
use tokio::sync::mpsc::Sender;
use crate::event_to_action::event_to_action::{EventToAction};
use crate::stream_interface::events::{ChatEvent};
use crate::utils::run_on_stream::StreamItemReceiver;
use crate::system_input::system_input::{SystemInput};
use crate::system_input::enigo::enigo_system_input::EnigoSystemInput;
use crate::actions::action::Action;

pub struct TestEventToAction {
    controller: EnigoSystemInput,
    sender: Sender<Action>
}

// impl TestEventToAction {
//     pub fn new(sender: Sender<Action>) -> TestEventToAction {
//         TestEventToAction { controller: EnigoSystemInput::new(), sender }
//     }
// }

impl EventToAction for TestEventToAction {
    fn execute(&mut self, event: ChatEvent) -> Option<Action> {
        execute(event, &mut self.controller)
    }
}

#[async_trait]
impl StreamItemReceiver for TestEventToAction {
    type Item = ChatEvent;
    async fn receive(&mut self, event: ChatEvent) {
        let maybe_action = self.execute(event);
        match maybe_action {
            Some(action) => {
                println!("test_event_to_action::send_in_channel::({:?})", action);
                match self.sender.send(action).await {
                    Ok(_) => println!("test_event_to_action::send_ok"),
                    Err(e) => println!("test_event_to_action::send_error::{}", e)
                }
            },
            _ => ()
        }
    }
}

fn execute(_event: ChatEvent, _system_input: &mut impl SystemInput) -> Option<Action> {
    println!("test_event_to_action::map_to::(test action key raw 1)");
    Some(Action::KeyRawDown(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::{mock, predicate::*};
    use crate::stream_interface::events::ChatMessage;
    use crate::{mock_system_input};

    mock_system_input!();

    #[test]
    fn any_event_move_mouse_hundred_pixels_x_y() {
        let mut mock = MockSystemInput::new();
        mock.expect_move_mouse_of()
            .with(eq(100), eq(100))
            .once()
            .return_const(());

        execute(
            ChatEvent::Message(ChatMessage { name: "".to_string(), content: "".to_string(), is_mod: false }),
            &mut mock
        );
    }
}
