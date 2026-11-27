// journal.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The journald native protocol datagram for one record.
//!
//! Fields are `KEY=value\n`, except that a value containing a newline
//! must use the binary form `KEY\n<u64 little-endian length><value>\n`
//! (systemd's "Native Journal Protocol"). Writing every value through
//! [`push_field`] means no payload can end a field early and forge
//! another, such as `PRIORITY=0` or `SYSLOG_IDENTIFIER=sshd`.

use std::sync::LazyLock;

/// The program name, sent as `SYSLOG_IDENTIFIER` so `journalctl -t`
/// finds the records.
static IDENTIFIER: LazyLock<String> = LazyLock::new(|| {
    std::env::args_os()
        .next()
        .as_deref()
        .map(std::path::Path::new)
        .and_then(std::path::Path::file_name)
        .map_or_else(
            || "rlg".to_string(),
            |name| name.to_string_lossy().into_owned(),
        )
});

/// The syslog priority journald expects for an rlg level name.
pub(super) fn priority(level: &str) -> &'static [u8] {
    match level {
        "ERROR" | "FATAL" | "CRITICAL" => b"3",
        "WARN" => b"4",
        "INFO" => b"6",
        "DEBUG" | "TRACE" | "VERBOSE" => b"7",
        _ => b"5",
    }
}

/// The datagram for one record. One trailing newline of `payload`
/// (the record terminator) is not part of the message.
pub(super) fn datagram(level: &str, payload: &[u8]) -> Vec<u8> {
    let message = payload.strip_suffix(b"\n").unwrap_or(payload);
    let mut buf = Vec::with_capacity(message.len() + 64);
    push_field(&mut buf, b"PRIORITY", priority(level));
    push_field(&mut buf, b"SYSLOG_IDENTIFIER", IDENTIFIER.as_bytes());
    push_field(&mut buf, b"MESSAGE", message);
    buf
}

/// Appends one field, in the binary form when `value` holds a newline.
fn push_field(buf: &mut Vec<u8>, key: &[u8], value: &[u8]) {
    buf.extend_from_slice(key);
    if value.contains(&b'\n') {
        buf.push(b'\n');
        buf.extend_from_slice(&(value.len() as u64).to_le_bytes());
    } else {
        buf.push(b'=');
    }
    buf.extend_from_slice(value);
    buf.push(b'\n');
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Decodes a datagram the way journald does.
    fn decode(mut d: &[u8]) -> Vec<(String, Vec<u8>)> {
        let mut fields = Vec::new();
        while !d.is_empty() {
            let end = d
                .iter()
                .position(|&b| b == b'\n' || b == b'=')
                .unwrap();
            let key = String::from_utf8(d[..end].to_vec()).unwrap();
            if d[end] == b'=' {
                let rest = &d[end + 1..];
                let nl = rest.iter().position(|&b| b == b'\n').unwrap();
                fields.push((key, rest[..nl].to_vec()));
                d = &rest[nl + 1..];
            } else {
                let len_bytes: [u8; 8] =
                    d[end + 1..end + 9].try_into().unwrap();
                let len =
                    usize::try_from(u64::from_le_bytes(len_bytes))
                        .unwrap();
                let value = &d[end + 9..end + 9 + len];
                assert_eq!(d[end + 9 + len], b'\n');
                fields.push((key, value.to_vec()));
                d = &d[end + 10 + len..];
            }
        }
        fields
    }

    #[test]
    fn a_newline_in_the_message_cannot_forge_a_field() {
        let fields = decode(&datagram(
            "INFO",
            b"hi\nPRIORITY=0\nSYSLOG_IDENTIFIER=sshd\n",
        ));
        let keys: Vec<&str> =
            fields.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, ["PRIORITY", "SYSLOG_IDENTIFIER", "MESSAGE"]);
        assert_eq!(fields[0].1, b"6");
        assert_ne!(fields[1].1, b"sshd");
        assert_eq!(
            fields[2].1,
            b"hi\nPRIORITY=0\nSYSLOG_IDENTIFIER=sshd"
        );
    }

    #[test]
    fn a_plain_message_uses_the_text_form_without_the_terminator() {
        let d = datagram("ERROR", b"disk full\n");
        assert!(d.ends_with(b"\nMESSAGE=disk full\n"), "{d:?}");
        assert!(d.starts_with(b"PRIORITY=3\n"), "{d:?}");
    }

    #[test]
    fn priorities_follow_syslog() {
        assert_eq!(priority("WARN"), b"4");
        assert_eq!(priority("TRACE"), b"7");
        assert_eq!(priority("FATAL"), b"3");
        assert_eq!(priority("NONE"), b"5");
    }
}
