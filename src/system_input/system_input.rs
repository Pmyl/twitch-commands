use enigo::{Enigo, MouseControllable};

pub struct SystemInput {
    enigo: Enigo,
}

impl SystemInput {
    pub fn new() -> SystemInput {
        SystemInput { enigo: Enigo::new() }
    }

    pub fn move_mouse_of(&mut self, x: i32, y: i32) {
        self.enigo.mouse_move_relative(x, y);
    }
}
