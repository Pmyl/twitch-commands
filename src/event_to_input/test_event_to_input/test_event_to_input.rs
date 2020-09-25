use async_trait::async_trait;
use crate::event_to_input::event_to_input::EventToInput;
use crate::stream_interface::events::{ChatEvents};
use crate::utils::run_on_stream::StreamItemReceiver;
use crate::system_input::system_input::{SystemInput};
use crate::system_input::enigo::enigo_system_input::EnigoSystemInput;

pub struct TestEventToInput {
    controller: EnigoSystemInput,
}

impl TestEventToInput {
    pub fn new() -> TestEventToInput {
        TestEventToInput { controller: EnigoSystemInput::new() }
    }
}

#[async_trait]
impl EventToInput for TestEventToInput {
    async fn execute(&mut self, event: ChatEvents) {
        execute(event, &mut self.controller).await;
    }
}

#[async_trait]
impl StreamItemReceiver for TestEventToInput {
    type Item = ChatEvents;
    async fn receive(&mut self, event: ChatEvents) {
        self.execute(event).await;
    }
}

async fn execute(_event: ChatEvents, system_input: &mut impl SystemInput) {
    println!("Mouse moved of {} {}", 100, 100);
    system_input.move_mouse_of(100, 100);
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::{mock, predicate::*};
    use crate::stream_interface::events::ChatMessage;
    use crate::{at, mock_system_input};

    mock_system_input!();

    #[test]
    fn any_event_move_mouse_hundred_pixels_x_y() {
        let mut mock = MockSystemInput::new();
        mock.expect_move_mouse_of()
            .with(eq(100), eq(100))
            .once()
            .return_const(());

        at!(execute(
            ChatEvents::Message(ChatMessage { name: "".to_string(), content: "".to_string(), is_mod: false }),
            &mut mock
        ));
    }
}