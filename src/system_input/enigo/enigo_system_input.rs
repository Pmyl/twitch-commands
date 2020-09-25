use enigo::{Enigo, MouseControllable, KeyboardControllable, Key};
use crate::system_input::system_input::SystemInput;
use tokio::time::{Duration, Delay, delay_for};

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

    fn arrow_up(&mut self) {
        self.enigo.key_click(Key::UpArrow);
    }

    fn arrow_down(&mut self) {
        self.enigo.key_click(Key::DownArrow);
    }

    fn delay_for(&mut self, ms: u64) -> Delay {
        delay_for(Duration::from_millis(ms))
    }
}
