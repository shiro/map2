use evdev_sys as raw;

pub enum LogPriority {
    /// critical errors and application bugs
    Error = raw::LIBEVDEV_LOG_ERROR as isize,
    /// informational messages
    Info = raw::LIBEVDEV_LOG_INFO as isize,
    /// debug information
    Debug = raw::LIBEVDEV_LOG_DEBUG as isize,
}

/// Define the minimum level to be printed to the log handler.
/// Messages higher than this level are printed, others are discarded. This
/// is a global setting and applies to any future logging messages.
pub fn set_log_priority(priority: LogPriority) {
    unsafe {
        raw::libevdev_set_log_priority(priority as i32);
    }
}

/// Return the current log priority level. Messages higher than this level
/// are printed, others are discarded. This is a global setting.
pub fn get_log_priority() -> LogPriority {
    unsafe {
        let priority = raw::libevdev_get_log_priority();
        match priority {
            raw::LIBEVDEV_LOG_ERROR => LogPriority::Error,
            raw::LIBEVDEV_LOG_INFO => LogPriority::Info,
            raw::LIBEVDEV_LOG_DEBUG => LogPriority::Debug,
            c => panic!("unknown log priority: {}", c),
        }
    }
}
