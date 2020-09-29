/// Quick to_string
/// e.g. let string: String = s!("test");
#[macro_export]
macro_rules! s {
    ($str:expr) => {
        $str.to_string()
    }
}
