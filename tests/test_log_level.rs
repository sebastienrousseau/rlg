// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(test)]

mod tests {
    use rlg::log_level::LogLevel;

    #[test]
    fn test_log_level_display() {
        assert_eq!(format!("{}", LogLevel::ALL), "ALL");
        assert_eq!(format!("{}", LogLevel::DEBUG), "DEBUG");
        assert_eq!(format!("{}", LogLevel::DISABLED), "DISABLED");
        assert_eq!(format!("{}", LogLevel::ERROR), "ERROR");
        assert_eq!(format!("{}", LogLevel::FATAL), "FATAL");
        assert_eq!(format!("{}", LogLevel::INFO), "INFO");
        assert_eq!(format!("{}", LogLevel::NONE), "NONE");
        assert_eq!(format!("{}", LogLevel::TRACE), "TRACE");
        assert_eq!(format!("{}", LogLevel::VERBOSE), "VERBOSE");
        assert_eq!(format!("{}", LogLevel::WARNING), "WARNING");
        assert_eq!(format!("{}", LogLevel::WARN), "WARN");
        assert_eq!(format!("{}", LogLevel::CRITICAL), "CRITICAL");
    }

    #[test]
    fn test_log_level_from_str() {
        assert_eq!("ALL".parse::<LogLevel>(), Ok(LogLevel::ALL));
        assert_eq!("DEBUG".parse::<LogLevel>(), Ok(LogLevel::DEBUG));
        assert_eq!("DISABLED".parse::<LogLevel>(), Ok(LogLevel::DISABLED));
        assert_eq!("ERROR".parse::<LogLevel>(), Ok(LogLevel::ERROR));
        assert_eq!("FATAL".parse::<LogLevel>(), Ok(LogLevel::FATAL));
        assert_eq!("INFO".parse::<LogLevel>(), Ok(LogLevel::INFO));
        assert_eq!("NONE".parse::<LogLevel>(), Ok(LogLevel::NONE));
        assert_eq!("TRACE".parse::<LogLevel>(), Ok(LogLevel::TRACE));
        assert_eq!("VERBOSE".parse::<LogLevel>(), Ok(LogLevel::VERBOSE));
        assert_eq!("WARNING".parse::<LogLevel>(), Ok(LogLevel::WARNING));
        assert_eq!("WARN".parse::<LogLevel>(), Ok(LogLevel::WARN));
        assert_eq!("CRITICAL".parse::<LogLevel>(), Ok(LogLevel::CRITICAL));
        assert_eq!(
            "Invalid".parse::<LogLevel>(),
            Err("Invalid log level: Invalid".to_string())
        );
    }

    #[test]
    fn test_log_level_try_from_str() {
        assert_eq!(TryInto::<LogLevel>::try_into("ALL"), Ok(LogLevel::ALL));
        assert_eq!(TryInto::<LogLevel>::try_into("DEBUG"), Ok(LogLevel::DEBUG));
        assert_eq!(
            TryInto::<LogLevel>::try_into("DISABLED"),
            Ok(LogLevel::DISABLED)
        );
        assert_eq!(TryInto::<LogLevel>::try_into("ERROR"), Ok(LogLevel::ERROR));
        assert_eq!(TryInto::<LogLevel>::try_into("FATAL"), Ok(LogLevel::FATAL));
        assert_eq!(TryInto::<LogLevel>::try_into("INFO"), Ok(LogLevel::INFO));
        assert_eq!(TryInto::<LogLevel>::try_into("NONE"), Ok(LogLevel::NONE));
        assert_eq!(TryInto::<LogLevel>::try_into("TRACE"), Ok(LogLevel::TRACE));
        assert_eq!(
            TryInto::<LogLevel>::try_into("VERBOSE"),
            Ok(LogLevel::VERBOSE)
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into("WARNING"),
            Ok(LogLevel::WARNING)
        );
        assert_eq!(TryInto::<LogLevel>::try_into("WARN"), Ok(LogLevel::WARN));
        assert_eq!(
            TryInto::<LogLevel>::try_into("CRITICAL"),
            Ok(LogLevel::CRITICAL)
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into("Invalid"),
            Err(LogLevel::INFO)
        );
    }

    #[test]
    fn test_log_level_try_from_string() {
        assert_eq!(
            TryInto::<LogLevel>::try_into(String::from("ALL")),
            Ok(LogLevel::ALL)
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into(String::from("DEBUG")),
            Ok(LogLevel::DEBUG)
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into(String::from("DISABLED")),
            Ok(LogLevel::DISABLED)
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into(String::from("ERROR")),
            Ok(LogLevel::ERROR)
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into(String::from("FATAL")),
            Ok(LogLevel::FATAL)
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into(String::from("INFO")),
            Ok(LogLevel::INFO)
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into(String::from("NONE")),
            Ok(LogLevel::NONE)
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into(String::from("TRACE")),
            Ok(LogLevel::TRACE)
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into(String::from("VERBOSE")),
            Ok(LogLevel::VERBOSE)
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into(String::from("WARNING")),
            Ok(LogLevel::WARNING)
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into(String::from("WARN")),
            Ok(LogLevel::WARN)
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into(String::from("CRITICAL")),
            Ok(LogLevel::CRITICAL)
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into(String::from("Invalid")),
            Err(LogLevel::INFO)
        );
    }
}
