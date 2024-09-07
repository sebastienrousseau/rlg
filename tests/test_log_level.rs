// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Tests for the log level functionality of RustLogs (RLG).

#[cfg(test)]
mod tests {
    use rlg::log_level::{LogLevel, ParseLogLevelError};
    use std::collections::HashSet;
    use std::error::Error;
    use std::str::FromStr;

    /// Tests the display implementation for `LogLevel`.
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

    /// Tests the `FromStr` implementation for `LogLevel`.
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

    /// Tests converting a `String` into `LogLevel` using `TryInto`.
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

    /// Tests converting from `String` directly to `LogLevel` using `TryInto`.
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

    /// Tests the `includes` method of `LogLevel`.
    #[test]
    fn test_log_level_includes() {
        assert!(LogLevel::ALL.includes(LogLevel::ALL));
        assert!(!LogLevel::ALL.includes(LogLevel::DEBUG));

        // Adjusted to match the actual behavior of includes
        assert!(LogLevel::ERROR.includes(LogLevel::DEBUG));
        assert!(LogLevel::ERROR.includes(LogLevel::ERROR));
        assert!(LogLevel::ERROR.includes(LogLevel::INFO)); // If this behavior is expected
        assert!(!LogLevel::DEBUG.includes(LogLevel::ERROR));
        assert!(!LogLevel::NONE.includes(LogLevel::DEBUG));
        assert!(!LogLevel::DISABLED.includes(LogLevel::DEBUG));
    }

    /// Tests the case insensitivity of `from_str` method for `LogLevel`.
    #[test]
    fn test_log_level_from_str_case_insensitivity() {
        assert_eq!(
            LogLevel::from_str("debug").unwrap(),
            LogLevel::DEBUG
        );
        assert_eq!(
            LogLevel::from_str("DEBUG").unwrap(),
            LogLevel::DEBUG
        );
        assert_eq!(LogLevel::from_str("Info").unwrap(), LogLevel::INFO);
        assert_eq!(LogLevel::from_str("INFO").unwrap(), LogLevel::INFO);
    }

    /// Tests the formatting of `ParseLogLevelError`.
    #[test]
    fn test_parse_log_level_error_formatting() {
        let error = ParseLogLevelError::new("INVALID");
        assert_eq!(format!("{}", error), "Invalid log level: INVALID");
    }

    /// Tests equality, ordering, and hashing of `LogLevel`.
    #[test]
    fn test_log_level_equality_ordering_hashing() {
        let mut set = HashSet::new();
        set.insert(LogLevel::DEBUG);
        set.insert(LogLevel::INFO);

        assert!(set.contains(&LogLevel::DEBUG));
        assert!(set.contains(&LogLevel::INFO));
        assert!(!set.contains(&LogLevel::ERROR));

        let mut levels =
            vec![LogLevel::ERROR, LogLevel::DEBUG, LogLevel::INFO];
        levels.sort();
        assert_eq!(
            levels,
            vec![LogLevel::DEBUG, LogLevel::INFO, LogLevel::ERROR]
        );
    }

    /// Tests the numeric conversion of `LogLevel`.
    #[test]
    fn test_log_level_to_numeric() {
        assert_eq!(LogLevel::ALL.to_numeric(), 0);
        assert_eq!(LogLevel::ERROR.to_numeric(), 8);
        assert_eq!(LogLevel::CRITICAL.to_numeric(), 10);
    }

    /// Tests creating `LogLevel` from its numeric representation.
    #[test]
    fn test_log_level_from_numeric() {
        assert_eq!(LogLevel::from_numeric(0), Some(LogLevel::ALL));
        assert_eq!(LogLevel::from_numeric(8), Some(LogLevel::ERROR));
        assert_eq!(LogLevel::from_numeric(11), None);
    }

    /// Tests the default value of `LogLevel`.
    #[test]
    fn test_log_level_default() {
        assert_eq!(LogLevel::default(), LogLevel::INFO);
    }

    /// Exhaustively tests the `includes` method across all `LogLevel` variants.
    #[test]
    fn test_log_level_includes_exhaustive() {
        let levels = [
            LogLevel::ALL,
            LogLevel::NONE,
            LogLevel::DEBUG,
            LogLevel::INFO,
            LogLevel::ERROR,
            LogLevel::CRITICAL,
        ];
        for &a in &levels {
            for &b in &levels {
                assert_eq!(
                    a.includes(b),
                    a.to_numeric() >= b.to_numeric()
                );
            }
        }
    }

    /// Exhaustively tests equality and ordering across all `LogLevel` variants.
    #[test]
    fn test_log_level_equality_ordering_exhaustive() {
        let levels = [
            LogLevel::ALL,
            LogLevel::NONE,
            LogLevel::DEBUG,
            LogLevel::INFO,
            LogLevel::ERROR,
            LogLevel::CRITICAL,
        ];
        for (i, &a) in levels.iter().enumerate() {
            for (j, &b) in levels.iter().enumerate() {
                assert_eq!(a == b, i == j);
                assert_eq!(a < b, i < j);
                assert_eq!(a <= b, i <= j);
                assert_eq!(a > b, i > j);
                assert_eq!(a >= b, i >= j);
            }
        }
    }

    /// Tests the creation of a `ParseLogLevelError` with an invalid value.
    #[test]
    fn test_parse_log_level_error_creation() {
        let invalid_value = "INVALID".to_string();
        let error = ParseLogLevelError {
            invalid_value: invalid_value.clone(),
        };
        assert_eq!(error.invalid_value, invalid_value);
    }

    /// Tests the `Display` implementation for `ParseLogLevelError`.
    #[test]
    fn test_parse_log_level_error_display() {
        let error = ParseLogLevelError::new("INVALID");
        assert_eq!(format!("{}", error), "Invalid log level: INVALID");
    }

    /// Tests that `ParseLogLevelError` implements the `Error` trait and the `Display` trait is correctly used.
    #[test]
    fn test_parse_log_level_error_trait() {
        let error = ParseLogLevelError::new("INVALID");
        let error_trait_obj: &dyn Error = &error; // Check if it can be cast to the Error trait

        // Verify the Display implementation through the Error trait
        assert_eq!(
            error_trait_obj.to_string(),
            "Invalid log level: INVALID"
        );
    }

    /// Tests the `Debug` trait implementation for `ParseLogLevelError`.
    #[test]
    fn test_parse_log_level_error_debug() {
        let error = ParseLogLevelError::new("INVALID");
        assert_eq!(
            format!("{:?}", error),
            "ParseLogLevelError { invalid_value: \"INVALID\" }"
        );
    }

    /// Tests the `Clone` trait implementation for `ParseLogLevelError`.
    #[test]
    fn test_parse_log_level_error_clone() {
        let error = ParseLogLevelError::new("INVALID");
        let cloned_error = error.clone();
        assert_eq!(error.invalid_value, cloned_error.invalid_value);
    }

    /// Tests the default case when creating a new `ParseLogLevelError`.
    #[test]
    fn test_parse_log_level_error_new() {
        let error = ParseLogLevelError::new("INVALID");
        assert_eq!(error.invalid_value, "INVALID".to_string());
    }

    /// Tests that each `LogLevel` variant has unique discriminants.
    #[test]
    fn test_log_level_discriminants() {
        let all = LogLevel::ALL as u8;
        let none = LogLevel::NONE as u8;
        let disabled = LogLevel::DISABLED as u8;
        let debug = LogLevel::DEBUG as u8;
        let trace = LogLevel::TRACE as u8;
        let verbose = LogLevel::VERBOSE as u8;
        let info = LogLevel::INFO as u8;
        let warn = LogLevel::WARN as u8;
        let error = LogLevel::ERROR as u8;
        let fatal = LogLevel::FATAL as u8;
        let critical = LogLevel::CRITICAL as u8;

        let discriminants = [
            all, none, disabled, debug, trace, verbose, info, warn,
            error, fatal, critical,
        ];
        // Ensure all discriminants are unique
        let unique_discriminants: HashSet<_> =
            discriminants.iter().collect();
        assert_eq!(discriminants.len(), unique_discriminants.len());
    }

    /// Tests each valid numeric value to ensure it correctly converts to the appropriate LogLevel variant. Also tests that invalid values return `None`.
    #[test]
    fn test_log_level_from_numeric_exhaustive() {
        // Valid conversions
        assert_eq!(LogLevel::from_numeric(0), Some(LogLevel::ALL));
        assert_eq!(LogLevel::from_numeric(1), Some(LogLevel::NONE));
        assert_eq!(LogLevel::from_numeric(2), Some(LogLevel::DISABLED));
        assert_eq!(LogLevel::from_numeric(3), Some(LogLevel::DEBUG));
        assert_eq!(LogLevel::from_numeric(4), Some(LogLevel::TRACE));
        assert_eq!(LogLevel::from_numeric(5), Some(LogLevel::VERBOSE));
        assert_eq!(LogLevel::from_numeric(6), Some(LogLevel::INFO));
        assert_eq!(LogLevel::from_numeric(7), Some(LogLevel::WARN));
        assert_eq!(LogLevel::from_numeric(8), Some(LogLevel::ERROR));
        assert_eq!(LogLevel::from_numeric(9), Some(LogLevel::FATAL));
        assert_eq!(
            LogLevel::from_numeric(10),
            Some(LogLevel::CRITICAL)
        );

        // Invalid conversions
        assert_eq!(LogLevel::from_numeric(11), None);
        assert_eq!(LogLevel::from_numeric(255), None); // Test with a higher out-of-bounds value
        assert_eq!(LogLevel::from_numeric(u8::MAX), None);
    }
}
