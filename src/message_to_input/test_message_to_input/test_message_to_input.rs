use async_trait::async_trait;
use crate::message_to_input::message_to_input::MessageToInput;
use crate::input_controller::input_controller::ControlInput;

pub struct TestMessageToInput {
    controller: ControlInput,
}

impl TestMessageToInput {
    pub fn new() -> TestMessageToInput {
        TestMessageToInput { controller: ControlInput::new() }
    }
}

#[async_trait]
impl MessageToInput for TestMessageToInput {
    async fn execute(&mut self, _message: String) {
        println!("Mouse moved of {} {}", 100, 100);
        self.controller.move_mouse_of(100, 100);
    }
}
