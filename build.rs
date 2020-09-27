extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/utils/is_mouse_pressed.cpp")
        .cpp(true)
        .compile("lib_mouse_pressed.a");
}
