// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(test)]

mod tests {
    use rlg::log_level::{LogLevel, ParseLogLevelError};
    use std::convert::TryInto;

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
        assert_eq!(format!("{}", LogLevel::WARN), "WARN");
        assert_eq!(format!("{}", LogLevel::CRITICAL), "CRITICAL");
    }

    #[test]
    fn test_log_level_from_str() {
        assert_eq!("ALL".parse::<LogLevel>().unwrap(), LogLevel::ALL);
        assert_eq!(
            "DEBUG".parse::<LogLevel>().unwrap(),
            LogLevel::DEBUG
        );
        assert_eq!(
            "DISABLED".parse::<LogLevel>().unwrap(),
            LogLevel::DISABLED
        );
        assert_eq!(
            "ERROR".parse::<LogLevel>().unwrap(),
            LogLevel::ERROR
        );
        assert_eq!(
            "FATAL".parse::<LogLevel>().unwrap(),
            LogLevel::FATAL
        );
        assert_eq!("INFO".parse::<LogLevel>().unwrap(), LogLevel::INFO);
        assert_eq!("NONE".parse::<LogLevel>().unwrap(), LogLevel::NONE);
        assert_eq!(
            "TRACE".parse::<LogLevel>().unwrap(),
            LogLevel::TRACE
        );
        assert_eq!(
            "VERBOSE".parse::<LogLevel>().unwrap(),
            LogLevel::VERBOSE
        );
        assert_eq!("WARN".parse::<LogLevel>().unwrap(), LogLevel::WARN);
        assert_eq!(
            "CRITICAL".parse::<LogLevel>().unwrap(),
            LogLevel::CRITICAL
        );

        match "Invalid".parse::<LogLevel>() {
            Err(ParseLogLevelError { .. }) => {} // Matches an error
            _ => panic!("Expected an error for invalid log level"),
        }
    }

    #[test]
    fn test_log_level_try_from_str() {
        assert_eq!(
            TryInto::<LogLevel>::try_into("ALL".to_string()).unwrap(),
            LogLevel::ALL
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into("DEBUG".to_string()).unwrap(),
            LogLevel::DEBUG
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into("DISABLED".to_string())
                .unwrap(),
            LogLevel::DISABLED
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into("ERROR".to_string()).unwrap(),
            LogLevel::ERROR
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into("FATAL".to_string()).unwrap(),
            LogLevel::FATAL
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into("INFO".to_string()).unwrap(),
            LogLevel::INFO
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into("NONE".to_string()).unwrap(),
            LogLevel::NONE
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into("TRACE".to_string()).unwrap(),
            LogLevel::TRACE
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into("VERBOSE".to_string())
                .unwrap(),
            LogLevel::VERBOSE
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into("WARN".to_string()).unwrap(),
            LogLevel::WARN
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into("CRITICAL".to_string())
                .unwrap(),
            LogLevel::CRITICAL
        );

        match TryInto::<LogLevel>::try_into("Invalid".to_string()) {
            Err(ParseLogLevelError { .. }) => {} // Matches an error
            _ => panic!("Expected an error for invalid log level"),
        }
    }

    #[test]
    fn test_log_level_try_from_string() {
        assert_eq!(
            TryInto::<LogLevel>::try_into(String::from("ALL")).unwrap(),
            LogLevel::ALL
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into(String::from("DEBUG"))
                .unwrap(),
            LogLevel::DEBUG
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into(String::from("DISABLED"))
                .unwrap(),
            LogLevel::DISABLED
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into(String::from("ERROR"))
                .unwrap(),
            LogLevel::ERROR
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into(String::from("FATAL"))
                .unwrap(),
            LogLevel::FATAL
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into(String::from("INFO"))
                .unwrap(),
            LogLevel::INFO
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into(String::from("NONE"))
                .unwrap(),
            LogLevel::NONE
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into(String::from("TRACE"))
                .unwrap(),
            LogLevel::TRACE
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into(String::from("VERBOSE"))
                .unwrap(),
            LogLevel::VERBOSE
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into(String::from("WARN"))
                .unwrap(),
            LogLevel::WARN
        );
        assert_eq!(
            TryInto::<LogLevel>::try_into(String::from("CRITICAL"))
                .unwrap(),
            LogLevel::CRITICAL
        );

        match TryInto::<LogLevel>::try_into(String::from("Invalid")) {
            Err(ParseLogLevelError { .. }) => {} // Matches an error
            _ => panic!("Expected an error for invalid log level"),
        }
    }
}
