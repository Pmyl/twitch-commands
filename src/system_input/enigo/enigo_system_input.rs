use enigo::{Enigo, MouseControllable};
use crate::system_input::system_input::SystemInput;

pub struct EnigoSystemInput {
    enigo: Enigo,
}

impl EnigoSystemInput {
    pub fn new() -> EnigoSystemInput {
        EnigoSystemInput { enigo: Enigo::new() }
    }
}

impl SystemInput for EnigoSystemInput {
    fn move_mouse_of(&mut self, x: i32, y: i32) {
        self.enigo.mouse_move_relative(x, y);
    }
}
