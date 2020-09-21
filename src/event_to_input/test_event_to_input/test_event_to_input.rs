use async_trait::async_trait;
use std::borrow::Borrow;
use crate::event_to_input::event_to_input::MessageToInput;
use crate::stream_interface::events::{ChatEvents};
use crate::utils::run_on_stream::StreamItemReceiver;
use crate::system_input::system_input::SystemInput;

pub struct TestMessageToInput {
    controller: SystemInput,
}

impl TestMessageToInput {
    pub fn new() -> TestMessageToInput {
        TestMessageToInput { controller: SystemInput::new() }
    }
}

#[async_trait]
impl MessageToInput for TestMessageToInput {
    async fn execute(&mut self, _event: ChatEvents) {
        println!("Mouse moved of {} {}", 100, 100);
        self.controller.move_mouse_of(100, 100);
    }
}

#[async_trait]
impl StreamItemReceiver for TestMessageToInput {
    type Item = ChatEvents;
    async fn receive(&mut self, event: ChatEvents) {
        self.execute(event).borrow();
    }
}
