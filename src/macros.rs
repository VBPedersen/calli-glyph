// macros.rs

/// Log at Trace level
#[macro_export]
macro_rules! log_trace {
    // Message + Context
    ($($arg:tt)*; $ctx:expr) => {
        $crate::core::debug::log(
            $crate::core::debug::LogLevel::Trace,
            format!($($arg)*),
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
    ($($arg:tt)*; $ctx:expr) => {
        $crate::core::debug::log(
            $crate::core::debug::LogLevel::Debug,
            format!($($arg)*),
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
    ($($arg:tt)*; $ctx:expr) => {
        $crate::core::debug::log(
            $crate::core::debug::LogLevel::Info,
            format!($($arg)*),
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
    ($($arg:tt)*; $ctx:expr) => {
        $crate::core::debug::log(
            $crate::core::debug::LogLevel::Warn,
            format!($($arg)*),
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
    ($($arg:tt)*; $ctx:expr) => {
        $crate::core::debug::log(
            $crate::core::debug::LogLevel::Error,
            format!($($arg)*),
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
