extern crate libc;

extern {
    fn is_mouse_pressed_cpp() -> libc::c_int;
}

pub fn is_mouse_pressed() -> bool {
    unsafe { is_mouse_pressed_cpp() == 1 }
}
