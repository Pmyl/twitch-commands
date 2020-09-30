use crate::system_input::system_input::SystemInput;
use tokio::time::{Duration, Delay, delay_for};
use std::mem::{transmute_copy, size_of, transmute};
use winapi::um::winuser::*;
use winapi::ctypes::{c_int, c_ulong};

pub struct CustomSystemInput {
}

impl CustomSystemInput {
    pub fn new() -> CustomSystemInput {
        CustomSystemInput { }
    }
}

impl SystemInput for CustomSystemInput {
    fn is_mouse_left_down(&self) -> bool {
        unsafe { GetKeyState(VK_LBUTTON) & 0x80 != 0 }
    }

    fn move_mouse_of(&mut self, x: i32, y: i32) {
        mouse_event(MOUSEEVENTF_MOVE, 0, x, y);
    }

    fn delay_for(&mut self, ms: u64) -> Delay {
        delay_for(Duration::from_millis(ms))
    }

    fn key_down(&mut self, raw: u16) {
        let extended_flag = extended_flag_if_necessary(raw);
        keybd_event(KEYEVENTF_SCANCODE | extended_flag, 0, key_to_scancode(raw));
    }

    fn key_up(&mut self, raw: u16) {
        let extended_flag = extended_flag_if_necessary(raw);
        keybd_event(KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP | extended_flag, 0, key_to_scancode(raw));
    }
}

fn extended_flag_if_necessary(virtual_keycode: u16) -> c_ulong {
    match virtual_keycode {
        n if n >= 37 && n <= 40 => KEYEVENTF_EXTENDEDKEY,
        _ => 0
    }
}

fn key_to_scancode(virtual_keycode: u16) -> u16 {
    unsafe { MapVirtualKeyW(virtual_keycode as u32, 0) as u16 }
}

fn keybd_event(flags: u32, vk: u16, scan: u16) {
    let mut input = INPUT {
        type_: INPUT_KEYBOARD,
        u: unsafe {
            transmute_copy(&KEYBDINPUT {
                wVk: vk,
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            })
        },
    };
    unsafe { SendInput(1, &mut input as LPINPUT, size_of::<INPUT>() as c_int) };
}

fn mouse_event(flags: u32, data: u32, dx: i32, dy: i32) {
    let mut input = INPUT {
        type_: INPUT_MOUSE,
        u: unsafe {
            transmute(MOUSEINPUT {
                dx,
                dy,
                mouseData: data,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            })
        },
    };
    unsafe { SendInput(1, &mut input as LPINPUT, size_of::<INPUT>() as c_int) };
}
