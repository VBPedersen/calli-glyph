// macros.rs

/// Log at Trace level
#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        $crate::core::debug::debug_log(
            $crate::core::debug::LogLevel::Trace,
            format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::core::debug::debug_log(
            $crate::core::debug::LogLevel::Debug,
            format!($($arg)*))
    };
}

/// Log at Info level
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::core::debug::debug_log(
            $crate::core::debug::LogLevel::Info,
            format!($($arg)*)
        )
    };
}

/// Log at Warn level
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::core::debug::debug_log(
            $crate::core::debug::LogLevel::Warn,
            format!($($arg)*))
    };
}

/// Log at Error level
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::core::debug::debug_log(
            $crate::core::debug::LogLevel::Error,
            format!($($arg)*))
    };
}
