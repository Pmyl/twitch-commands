use tokio::time::Delay;

pub trait SystemInput {
    fn is_mouse_left_down(&self) -> bool;
    fn move_mouse_of(&mut self, x: i32, y: i32);
    fn delay_for(&mut self, ms: u64) -> Delay;
    fn key_down(&mut self, raw: u16);
    fn key_up(&mut self, raw: u16);
}
