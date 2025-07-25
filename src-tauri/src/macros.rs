#![macro_use]

#[cfg(debug_assertions)]
#[inline(always)]
pub fn debug_print(args: std::fmt::Arguments) {
    println!("{}", args);
}

#[cfg(not(debug_assertions))]
#[inline(always)]
pub fn debug_print(_args: std::fmt::Arguments) {}

macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::macros::debug_print(format_args!($($arg)*));
    };
}
