// Copyright © 2024 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(test)]

mod tests {
    use rlg::log_format::LogFormat;

    #[test]
    fn test_log_format_display() {
        assert_eq!(format!("{}", LogFormat::CLF), "CLF");
        assert_eq!(format!("{}", LogFormat::JSON), "JSON");
        assert_eq!(format!("{}", LogFormat::CEF), "CEF");
        assert_eq!(format!("{}", LogFormat::ELF), "ELF");
        assert_eq!(format!("{}", LogFormat::W3C), "W3C");
        assert_eq!(format!("{}", LogFormat::GELF), "GELF");
        assert_eq!(
            format!("{}", LogFormat::ApacheAccessLog),
            "Apache Access Log"
        );
        assert_eq!(format!("{}", LogFormat::Logstash), "Logstash");
        assert_eq!(format!("{}", LogFormat::Log4jXML), "Log4j XML");
        assert_eq!(format!("{}", LogFormat::NDJSON), "NDJSON");
    }

    #[test]
    fn test_log_format_from_str() {
        assert_eq!("CLF".parse::<LogFormat>().unwrap(), LogFormat::CLF);
        assert_eq!(
            "JSON".parse::<LogFormat>().unwrap(),
            LogFormat::JSON
        );
        assert_eq!("CEF".parse::<LogFormat>().unwrap(), LogFormat::CEF);
        assert_eq!("ELF".parse::<LogFormat>().unwrap(), LogFormat::ELF);
        assert_eq!("W3C".parse::<LogFormat>().unwrap(), LogFormat::W3C);
        assert_eq!(
            "GELF".parse::<LogFormat>().unwrap(),
            LogFormat::GELF
        );
        assert_eq!(
            "ApacheAccessLog".parse::<LogFormat>().unwrap(),
            LogFormat::ApacheAccessLog
        );
        assert_eq!(
            "Logstash".parse::<LogFormat>().unwrap(),
            LogFormat::Logstash
        );
        assert_eq!(
            "Log4jXML".parse::<LogFormat>().unwrap(),
            LogFormat::Log4jXML
        );
        assert_eq!(
            "NDJSON".parse::<LogFormat>().unwrap(),
            LogFormat::NDJSON
        );
        assert!("Invalid".parse::<LogFormat>().is_err());
    }
}
