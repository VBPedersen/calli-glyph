// macros.rs

/// Log at Trace level
#[macro_export]
macro_rules! log_trace {
    // Message + Context
    ($fmt:expr $(, $arg:expr)*; $ctx:expr) => {
        $crate::core::debug::log(
            $crate::core::debug::LogLevel::Trace,
            format!($fmt $(, $arg)*),
            Some($ctx.to_string())
        )
    };
    // Message only
    ($($arg:tt)*) => {
        $crate::core::debug::log(
            $crate::core::debug::LogLevel::Trace,
            format!($($arg)*),
            None
        )
    };
}

#[macro_export]
macro_rules! log_debug {
    ($fmt:expr $(, $arg:expr)*; $ctx:expr) => {
        $crate::core::debug::log(
            $crate::core::debug::LogLevel::Debug,
            format!($fmt $(, $arg)*),
            Some($ctx.to_string())
        )
    };

    ($($arg:tt)*) => {
        $crate::core::debug::log(
            $crate::core::debug::LogLevel::Debug,
            format!($($arg)*),
            None)
    };
}

/// Log at Info level
#[macro_export]
macro_rules! log_info {
    ($fmt:expr $(, $arg:expr)*; $ctx:expr) => {
        $crate::core::debug::log(
            $crate::core::debug::LogLevel::Info,
            format!($fmt $(, $arg)*),
            Some($ctx.to_string())
        )
    };

    ($($arg:tt)*) => {
        $crate::core::debug::log(
            $crate::core::debug::LogLevel::Info,
            format!($($arg)*),
            None)
    };
}

/// Log at Warn level
#[macro_export]
macro_rules! log_warn {
    ($fmt:expr $(, $arg:expr)*; $ctx:expr) => {
        $crate::core::debug::log(
            $crate::core::debug::LogLevel::Warn,
            format!($fmt $(, $arg)*),
            Some($ctx.to_string())
        )
    };

    ($($arg:tt)*) => {
        $crate::core::debug::log(
            $crate::core::debug::LogLevel::Warn,
            format!($($arg)*),
            None)
    };
}

/// Log at Error level
#[macro_export]
macro_rules! log_error {
    ($fmt:expr $(, $arg:expr)*; $ctx:expr) => {
        $crate::core::debug::log(
            $crate::core::debug::LogLevel::Error,
            format!($fmt $(, $arg)*),
            Some($ctx.to_string())
        )
    };

    ($($arg:tt)*) => {
        $crate::core::debug::log(
            $crate::core::debug::LogLevel::Error,
            format!($($arg)*),
            None)
    };
}
