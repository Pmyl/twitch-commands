use tokio::time::Delay;

pub trait SystemInput {
    fn move_mouse_of(&mut self, x: i32, y: i32);
    fn arrow_up(&mut self);
    fn arrow_down(&mut self);
    fn delay_for(&mut self, ms: u64) -> Delay;
}

#[cfg(test)]
#[macro_export]
macro_rules! mock_system_input {
    () => {
        use tokio::time::{Delay};

        mock! {
            SystemInput {}
            trait SystemInput {
                fn move_mouse_of(&mut self, x: i32, y: i32);
                fn arrow_up(&mut self);
                fn arrow_down(&mut self);
                fn delay_for(&mut self, ms: u64) -> Delay;
            }
        }
    };
}
