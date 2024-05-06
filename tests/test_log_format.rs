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
        assert_eq!("CLF".parse::<LogFormat>(), Ok(LogFormat::CLF));
        assert_eq!("JSON".parse::<LogFormat>(), Ok(LogFormat::JSON));
        assert_eq!("CEF".parse::<LogFormat>(), Ok(LogFormat::CEF));
        assert_eq!("ELF".parse::<LogFormat>(), Ok(LogFormat::ELF));
        assert_eq!("W3C".parse::<LogFormat>(), Ok(LogFormat::W3C));
        assert_eq!("GELF".parse::<LogFormat>(), Ok(LogFormat::GELF));
        assert_eq!(
            "Apache Access Log".parse::<LogFormat>(),
            Ok(LogFormat::ApacheAccessLog)
        );
        assert_eq!("Logstash".parse::<LogFormat>(), Ok(LogFormat::Logstash));
        assert_eq!("Log4j XML".parse::<LogFormat>(), Ok(LogFormat::Log4jXML));
        assert_eq!("NDJSON".parse::<LogFormat>(), Ok(LogFormat::NDJSON));
        assert_eq!(
            "Invalid".parse::<LogFormat>(),
            Err("Invalid log format: Invalid".to_string())
        );
    }

    #[test]
    fn test_log_format_try_from_str() {
        assert_eq!(TryInto::<LogFormat>::try_into("CLF"), Ok(LogFormat::CLF));
        assert_eq!(TryInto::<LogFormat>::try_into("JSON"), Ok(LogFormat::JSON));
        assert_eq!(TryInto::<LogFormat>::try_into("CEF"), Ok(LogFormat::CEF));
        assert_eq!(TryInto::<LogFormat>::try_into("ELF"), Ok(LogFormat::ELF));
        assert_eq!(TryInto::<LogFormat>::try_into("W3C"), Ok(LogFormat::W3C));
        assert_eq!(TryInto::<LogFormat>::try_into("GELF"), Ok(LogFormat::GELF));
        assert_eq!(
            TryInto::<LogFormat>::try_into("Apache Access Log"),
            Ok(LogFormat::ApacheAccessLog)
        );
        assert_eq!(
            TryInto::<LogFormat>::try_into("Logstash"),
            Ok(LogFormat::Logstash)
        );
        assert_eq!(
            TryInto::<LogFormat>::try_into("Log4j XML"),
            Ok(LogFormat::Log4jXML)
        );
        assert_eq!(
            TryInto::<LogFormat>::try_into("NDJSON"),
            Ok(LogFormat::NDJSON)
        );
        assert_eq!(
            TryInto::<LogFormat>::try_into("Invalid"),
            Err(LogFormat::default())
        );
    }

    #[test]
    fn test_log_format_try_from_string() {
        assert_eq!(
            TryInto::<LogFormat>::try_into(String::from("CLF")),
            Ok(LogFormat::CLF)
        );
        assert_eq!(
            TryInto::<LogFormat>::try_into(String::from("JSON")),
            Ok(LogFormat::JSON)
        );
        assert_eq!(
            TryInto::<LogFormat>::try_into(String::from("CEF")),
            Ok(LogFormat::CEF)
        );
        assert_eq!(
            TryInto::<LogFormat>::try_into(String::from("ELF")),
            Ok(LogFormat::ELF)
        );
        assert_eq!(
            TryInto::<LogFormat>::try_into(String::from("W3C")),
            Ok(LogFormat::W3C)
        );
        assert_eq!(
            TryInto::<LogFormat>::try_into(String::from("GELF")),
            Ok(LogFormat::GELF)
        );
        assert_eq!(
            TryInto::<LogFormat>::try_into(String::from("Apache Access Log")),
            Ok(LogFormat::ApacheAccessLog)
        );
        assert_eq!(
            TryInto::<LogFormat>::try_into(String::from("Logstash")),
            Ok(LogFormat::Logstash)
        );
        assert_eq!(
            TryInto::<LogFormat>::try_into(String::from("Log4j XML")),
            Ok(LogFormat::Log4jXML)
        );
        assert_eq!(
            TryInto::<LogFormat>::try_into(String::from("NDJSON")),
            Ok(LogFormat::NDJSON)
        );
        assert_eq!(
            TryInto::<LogFormat>::try_into(String::from("Invalid")),
            Err(LogFormat::default())
        );
    }
}
