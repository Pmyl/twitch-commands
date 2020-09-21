use enigo::{Enigo, MouseControllable};

pub struct ControlInput {
    enigo: Enigo,
}

impl ControlInput {
    pub fn new() -> ControlInput {
        ControlInput { enigo: Enigo::new() }
    }

    pub fn move_mouse_of(&mut self, x: i32, y: i32) {
        self.enigo.mouse_move_relative(x, y);
    }
}