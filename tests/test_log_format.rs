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

    #[test]
    fn test_log_format_validate() {
        assert!(LogFormat::CLF.validate(
        "127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326"
    ));
        assert!(LogFormat::JSON.validate("{\"key\":\"value\"}"));
        assert!(LogFormat::CEF.validate("CEF:0|security|threat|1.0|100|Something happened|5|msg=hello"));
        assert!(LogFormat::W3C.validate("#Fields: date time c-ip cs-method cs-uri-stem sc-status\n2024-01-01 12:34:56 192.168.0.1 GET /index.html 200"));
        assert!(LogFormat::GELF.validate("{\"version\":\"1.1\",\"host\":\"localhost\",\"short_message\":\"A short message\"}"));
        assert!(LogFormat::Log4jXML.validate("<log4j:event logger=\"myLogger\" timestamp=\"1234567890\">"));

        // Invalid cases
        assert!(!LogFormat::CLF.validate("Invalid CLF log"));
        assert!(!LogFormat::JSON.validate("Invalid JSON"));
        assert!(!LogFormat::CEF.validate("Invalid CEF log"));
        assert!(!LogFormat::W3C.validate("Invalid W3C log"));
        assert!(!LogFormat::GELF.validate("Invalid GELF log"));
        assert!(!LogFormat::Log4jXML.validate("<invalid>XML</invalid>"));
    }

    #[test]
    fn test_log_format_format_log() {
        // Valid formatting
        assert_eq!(
            LogFormat::CLF
                .format_log(
                    "127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326"
                )
                .unwrap(),
            "127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326"
        );

        assert_eq!(
            LogFormat::JSON.format_log("{\"key\":\"value\"}").unwrap(),
            "{\n  \"key\": \"value\"\n}"
        );

        assert_eq!(
            LogFormat::CEF
                .format_log("CEF:0|security|threat|1.0|100|Something happened|5|msg=hello")
                .unwrap(),
            "CEF:0|security|threat|1.0|100|Something happened|5|msg=hello"
        );

        assert_eq!(
            LogFormat::Log4jXML
                .format_log("<log4j:event logger=\"myLogger\" timestamp=\"1234567890\">")
                .unwrap(),
            "<log4j:event logger=\"myLogger\" timestamp=\"1234567890\">"
        );

        // Invalid JSON
        let invalid_json = "Invalid JSON";
        assert!(LogFormat::JSON.format_log(invalid_json).is_err());
    }
}
