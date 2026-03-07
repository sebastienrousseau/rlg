#![cfg(not(miri))]
#![allow(missing_docs)]
#[cfg(test)]
mod tests {
    use rlg::config::{ConfigError, LogRotation};
    use std::num::NonZeroU64;
    use std::str::FromStr;

    #[test]
    fn log_rotation_from_str_valid_inputs() {
        let test_cases = vec![
            (
                "size:1024",
                LogRotation::Size(NonZeroU64::new(1024).unwrap()),
            ),
            (
                "time:60",
                LogRotation::Time(NonZeroU64::new(60).unwrap()),
            ),
            ("date", LogRotation::Date),
            ("count:5", LogRotation::Count(5)),
            (
                "SIZE:512",
                LogRotation::Size(NonZeroU64::new(512).unwrap()),
            ),
        ];

        for (input, expected) in test_cases {
            let result = LogRotation::from_str(input).unwrap();
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn log_rotation_from_str_invalid_inputs() {
        let test_cases = vec![
            ("invalid", "Invalid log rotation option: 'invalid'"),
            ("size:0", "Log rotation size must be greater than 0"),
            ("time:0", "Log rotation time must be greater than 0"),
            ("time:abc", "Invalid time value for log rotation: 'abc'"),
            ("count:0", "Log rotation count must be greater than 0"),
            ("count:", "Invalid count value for log rotation: ''"),
            ("size:", "Invalid size value for log rotation: ''"),
        ];

        for (input, expected_err) in test_cases {
            let result = LogRotation::from_str(input);
            match result {
                Err(ConfigError::ValidationError(msg)) => assert_eq!(
                    msg, expected_err,
                    "Failed for input: {}",
                    input
                ),
                _ => panic!(
                    "Expected ValidationError for input: {}",
                    input
                ),
            }
        }
    }

    #[test]
    fn log_rotation_display() {
        assert_eq!(
            format!("{}", LogRotation::Date),
            "Date-based rotation"
        );
        assert_eq!(
            format!(
                "{}",
                LogRotation::Size(NonZeroU64::new(100).unwrap())
            ),
            "Size: 100 bytes"
        );
        assert_eq!(
            format!(
                "{}",
                LogRotation::Time(NonZeroU64::new(50).unwrap())
            ),
            "Time: 50 seconds"
        );
        assert_eq!(
            format!("{}", LogRotation::Count(10)),
            "Count: 10 logs"
        );
    }
}
