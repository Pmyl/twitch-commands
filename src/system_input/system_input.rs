pub trait SystemInput {
    fn move_mouse_of(&mut self, x: i32, y: i32);
    fn arrow_up(&mut self);
    fn arrow_down(&mut self);
}


#[cfg(test)]
#[macro_export]
macro_rules! mock_system_input {
    () => {
        mock! {
            SystemInput {}
            trait SystemInput {
                fn move_mouse_of(&mut self, x: i32, y: i32);
                fn arrow_up(&mut self);
                fn arrow_down(&mut self);
            }
        }
    }
}
