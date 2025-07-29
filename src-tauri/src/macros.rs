#[cfg(debug_assertions)]
#[macro_export]
macro_rules! debug {
    (green, $($arg:tt)*) => {
            println!("\x1b[32m{}\x1b[0m", format_args!($($arg)*))
    };
    (yellow, $($arg:tt)*) => {
            println!("\x1b[33m{}\x1b[0m", format_args!($($arg)*))
    };
    (red, $($arg:tt)*) => {
            println!("\x1b[31m{}\x1b[0m", format_args!($($arg)*))
    };
    (dim, $($arg:tt)*) => {
            println!("\x1b[90m{}\x1b[0m", format_args!($($arg)*))
    };
    // Debug mode: Expand to println!
    ($($arg:tt)*) => {
            println!($($arg)*)
    };
}
#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! debug {
    // no op
    ($($arg:tt)*) => {{}};
}
