// format.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

/// Which of the 14 output formats a [`Record`](crate::Record) is
/// written in.
///
/// The variants and their order mirror `rlg::LogFormat`, which stays
/// in `rlg` because its parsing and validation need `std`. The
/// discriminant indexes the writer table in [`crate::record`], so the
/// order is load-bearing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Format {
    /// Common Log Format (CLF)
    CLF,
    /// JavaScript Object Notation (JSON)
    JSON,
    /// Common Event Format (CEF)
    CEF,
    /// Extended Log Format (ELF)
    ELF,
    /// W3C Extended Log Format (W3C)
    W3C,
    /// Graylog Extended Log Format (GELF)
    GELF,
    /// Apache Access Log Format
    ApacheAccessLog,
    /// Logstash Format
    Logstash,
    /// Log4j XML Format
    Log4jXML,
    /// Network Data JSON (NDJSON)
    NDJSON,
    /// Model Context Protocol (MCP) - AI Native
    MCP,
    /// OpenTelemetry Logging (OTLP) - AI Native
    OTLP,
    /// Logfmt (key=value)
    Logfmt,
    /// Elastic Common Schema (ECS)
    ECS,
}

impl Format {
    /// Every format, in declaration order.
    pub const ALL: [Self; 14] = [
        Self::CLF,
        Self::JSON,
        Self::CEF,
        Self::ELF,
        Self::W3C,
        Self::GELF,
        Self::ApacheAccessLog,
        Self::Logstash,
        Self::Log4jXML,
        Self::NDJSON,
        Self::MCP,
        Self::OTLP,
        Self::Logfmt,
        Self::ECS,
    ];

    /// The position of this format in [`Format::ALL`].
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An exhaustive match: a new variant fails to compile here until
    /// it is placed in `ALL` (and so in the writer table).
    const fn position(f: Format) -> usize {
        match f {
            Format::CLF => 0,
            Format::JSON => 1,
            Format::CEF => 2,
            Format::ELF => 3,
            Format::W3C => 4,
            Format::GELF => 5,
            Format::ApacheAccessLog => 6,
            Format::Logstash => 7,
            Format::Log4jXML => 8,
            Format::NDJSON => 9,
            Format::MCP => 10,
            Format::OTLP => 11,
            Format::Logfmt => 12,
            Format::ECS => 13,
        }
    }

    #[test]
    fn all_is_in_declaration_order() {
        for (i, f) in Format::ALL.iter().enumerate() {
            assert_eq!(f.index(), i);
            assert_eq!(position(*f), i);
        }
    }
}
