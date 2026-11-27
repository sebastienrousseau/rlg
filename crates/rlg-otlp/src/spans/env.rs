// env.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! The OpenTelemetry environment variables `SpanExporter` reads.

/// Environment lookups for `SpanExporterBuilder::from_lookup`,
/// with empty values treated as unset.
pub(super) struct Env<'a, F: Fn(&str) -> Option<String>>(
    pub(super) &'a F,
);

impl<F: Fn(&str) -> Option<String>> Env<'_, F> {
    pub(super) fn get(&self, key: &str) -> Option<String> {
        (self.0)(key).filter(|v| !v.trim().is_empty())
    }

    pub(super) fn endpoint(&self) -> Option<String> {
        self.get("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT").or_else(|| {
            self.get("OTEL_EXPORTER_OTLP_ENDPOINT").map(|base| {
                format!("{}/v1/traces", base.trim_end_matches('/'))
            })
        })
    }

    /// The `key=value` pairs of the headers variable, values
    /// percent-decoded as the OTLP exporter spec requires. A pair
    /// without `=`, a bad escape or a value that is not UTF-8 is an
    /// error, reported when the exporter is built.
    pub(super) fn headers(
        &self,
    ) -> Result<Vec<(String, String)>, String> {
        let raw = self
            .get("OTEL_EXPORTER_OTLP_TRACES_HEADERS")
            .or_else(|| self.get("OTEL_EXPORTER_OTLP_HEADERS"))
            .unwrap_or_default();
        raw.split(',')
            .filter(|kv| !kv.trim().is_empty())
            .map(|kv| {
                let (k, v) = kv.split_once('=').ok_or_else(|| {
                    format!("OTLP headers: {kv:?} is not key=value")
                })?;
                Ok((k.trim().to_string(), percent_decode(v.trim())?))
            })
            .collect()
    }
}

/// Decodes `%XX` escapes; the result must be UTF-8.
fn percent_decode(s: &str) -> Result<String, String> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let byte = s
                .get(i + 1..i + 3)
                .and_then(|hex| u8::from_str_radix(hex, 16).ok())
                .ok_or_else(|| {
                    format!("OTLP headers: bad escape in {s:?}")
                })?;
            out.push(byte);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).map_err(|_| {
        format!("OTLP headers: {s:?} does not decode to UTF-8")
    })
}

#[cfg(test)]
mod tests {
    use super::percent_decode;

    #[test]
    fn decodes_escapes_and_refuses_bad_ones() {
        assert_eq!(percent_decode("a%20b%2C%3d").unwrap(), "a b,=");
        assert_eq!(percent_decode("caf%C3%A9").unwrap(), "caf\u{e9}");
        for bad in ["%", "%2", "%zz", "%C3"] {
            assert!(percent_decode(bad).is_err(), "{bad}");
        }
    }
}
