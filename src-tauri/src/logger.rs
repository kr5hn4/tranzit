// log with file name and line numbers in debug build
#[cfg(debug_assertions)]
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        println!("[{}:{}] {}", file!(), line!(), format!($($arg)*))
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        () // no-op in release builds
    };
}
