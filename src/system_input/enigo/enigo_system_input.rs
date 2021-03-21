use enigo::{Enigo, MouseControllable, KeyboardControllable, Key};
use crate::system_input::system_input::SystemInput;
use tokio::time::{Duration, sleep, Sleep};

pub struct EnigoSystemInput {
    enigo: Enigo,
}

impl EnigoSystemInput {
    #[allow(dead_code)]
    pub fn new() -> EnigoSystemInput {
        EnigoSystemInput { enigo: Enigo::new() }
    }
}

impl SystemInput for EnigoSystemInput {
    fn is_mouse_left_down(&self) -> bool {
        unimplemented!()
    }

    fn move_mouse_of(&mut self, x: i32, y: i32) {
        self.enigo.mouse_move_relative(x, y);
    }

    fn delay_for(&mut self, ms: u64) -> Sleep {
        sleep(Duration::from_millis(ms))
    }

    fn key_down(&mut self, raw: u16) {
        self.enigo.key_down(Key::Raw(raw))
    }

    fn key_up(&mut self, raw: u16) {
        self.enigo.key_up(Key::Raw(raw))
    }
}
