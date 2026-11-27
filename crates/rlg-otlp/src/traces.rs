// traces.rs
// Copyright © 2024-2026 RustLogs (RLG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! OTLP/HTTP JSON encoding of closed spans (`/v1/traces`).

use rlg::tracing::{SpanKind, SpanRecord, SpanStatus};
use serde_json::{Value, json};

/// The endpoint a [`SpanExporter`](crate::SpanExporter) uses when
/// given none: the OTLP/HTTP traces route of a Collector on this host.
pub const DEFAULT_TRACES_ENDPOINT: &str =
    "http://localhost:4318/v1/traces";

/// Serialise closed spans into one OTLP/HTTP JSON
/// `ExportTraceServiceRequest`, under a resource whose
/// `service.name` is `service_name`.
#[must_use]
pub fn serialise_spans(
    spans: &[SpanRecord],
    service_name: &str,
) -> String {
    let spans: Vec<Value> = spans.iter().map(span_json).collect();
    json!({
        "resourceSpans": [{
            "resource": {
                "attributes": [key_value("service.name", &json!(service_name))]
            },
            "scopeSpans": [{
                "scope": {
                    "name": "rlg-otlp",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "spans": spans
            }]
        }]
    })
    .to_string()
}

fn span_json(span: &SpanRecord) -> Value {
    let attributes: Vec<Value> = span
        .attributes
        .iter()
        .map(|(k, v)| key_value(k, v))
        .chain(std::iter::once(key_value(
            "code.namespace",
            &json!(span.target),
        )))
        .collect();
    let mut out = json!({
        "traceId": span.trace_id,
        "spanId": span.span_id,
        "name": span.name,
        "kind": kind_code(span.kind),
        // OTLP JSON carries 64-bit integers as strings.
        "startTimeUnixNano": span.start_unix_nanos.to_string(),
        "endTimeUnixNano": span.end_unix_nanos.to_string(),
        "attributes": attributes,
        "status": status_json(&span.status),
    });
    if let Some(parent) = &span.parent_span_id {
        out["parentSpanId"] = json!(parent);
    }
    out
}

fn key_value(key: &str, value: &Value) -> Value {
    json!({ "key": key, "value": any_value(value) })
}

/// A JSON value as an OTLP `AnyValue`.
fn any_value(value: &Value) -> Value {
    match value {
        Value::String(s) => json!({ "stringValue": s }),
        Value::Bool(b) => json!({ "boolValue": b }),
        Value::Number(n) if n.is_i64() || n.is_u64() => {
            json!({ "intValue": n.to_string() })
        }
        Value::Number(n) => json!({ "doubleValue": n.as_f64() }),
        Value::Array(items) => {
            let values: Vec<Value> =
                items.iter().map(any_value).collect();
            json!({ "arrayValue": { "values": values } })
        }
        Value::Null | Value::Object(_) => {
            json!({ "stringValue": value.to_string() })
        }
    }
}

const fn kind_code(kind: SpanKind) -> u8 {
    match kind {
        SpanKind::Internal => 1,
        SpanKind::Server => 2,
        SpanKind::Client => 3,
        SpanKind::Producer => 4,
        SpanKind::Consumer => 5,
    }
}

fn status_json(status: &SpanStatus) -> Value {
    match status {
        SpanStatus::Unset => json!({ "code": 0 }),
        SpanStatus::Ok => json!({ "code": 1 }),
        SpanStatus::Error(message) => {
            json!({ "code": 2, "message": message })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn record() -> SpanRecord {
        let mut attributes = BTreeMap::new();
        let _ =
            attributes.insert("route".into(), json!("/v1/allocate"));
        let _ = attributes.insert("status".into(), json!(200));
        let _ = attributes.insert("ratio".into(), json!(0.5));
        let _ = attributes.insert("cached".into(), json!(false));
        let _ = attributes.insert("assets".into(), json!(["a", 1]));
        let _ = attributes.insert("extra".into(), json!({"k": null}));
        SpanRecord {
            trace_id: "0af7651916cd43dd8448eb211c80319c".into(),
            span_id: "b7ad6b7169203331".into(),
            parent_span_id: Some("00f067aa0ba902b7".into()),
            name: "request",
            target: "nalufx::api",
            start_unix_nanos: 1_000,
            end_unix_nanos: 2_000,
            attributes,
            kind: SpanKind::Server,
            status: SpanStatus::Error("boom".into()),
        }
    }

    fn first_span(body: &str) -> Value {
        let v: Value = serde_json::from_str(body).unwrap();
        v["resourceSpans"][0]["scopeSpans"][0]["spans"][0].clone()
    }

    fn attribute(span: &Value, key: &str) -> Value {
        span["attributes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|kv| kv["key"] == key)
            .unwrap()["value"]
            .clone()
    }

    #[test]
    fn encodes_ids_times_kind_and_status() {
        let body = serialise_spans(&[record()], "nalufx");
        let v: Value = serde_json::from_str(&body).unwrap();
        let resource =
            &v["resourceSpans"][0]["resource"]["attributes"][0];
        assert_eq!(resource["value"]["stringValue"], "nalufx");
        let span = first_span(&body);
        assert_eq!(span["traceId"], "0af7651916cd43dd8448eb211c80319c");
        assert_eq!(span["spanId"], "b7ad6b7169203331");
        assert_eq!(span["parentSpanId"], "00f067aa0ba902b7");
        assert_eq!(span["name"], "request");
        assert_eq!(span["kind"], 2);
        assert_eq!(span["startTimeUnixNano"], "1000");
        assert_eq!(span["endTimeUnixNano"], "2000");
        assert_eq!(
            span["status"],
            json!({"code": 2, "message": "boom"})
        );
    }

    #[test]
    fn encodes_attribute_types() {
        let span = first_span(&serialise_spans(&[record()], "s"));
        assert_eq!(
            attribute(&span, "route"),
            json!({"stringValue": "/v1/allocate"})
        );
        assert_eq!(
            attribute(&span, "status"),
            json!({"intValue": "200"})
        );
        assert_eq!(
            attribute(&span, "ratio"),
            json!({"doubleValue": 0.5})
        );
        assert_eq!(
            attribute(&span, "cached"),
            json!({"boolValue": false})
        );
        assert_eq!(
            attribute(&span, "assets"),
            json!({"arrayValue": {"values": [{"stringValue": "a"}, {"intValue": "1"}]}})
        );
        assert_eq!(
            attribute(&span, "extra"),
            json!({"stringValue": "{\"k\":null}"})
        );
        assert_eq!(
            attribute(&span, "code.namespace"),
            json!({"stringValue": "nalufx::api"})
        );
    }

    #[test]
    fn root_span_has_no_parent_and_unset_status() {
        let mut root = record();
        root.parent_span_id = None;
        root.status = SpanStatus::Unset;
        root.kind = SpanKind::Internal;
        let span = first_span(&serialise_spans(&[root], "s"));
        assert!(span.get("parentSpanId").is_none());
        assert_eq!(span["status"], json!({"code": 0}));
        assert_eq!(span["kind"], 1);
        let mut ok = record();
        ok.status = SpanStatus::Ok;
        ok.kind = SpanKind::Consumer;
        let span = first_span(&serialise_spans(&[ok], "s"));
        assert_eq!(span["status"], json!({"code": 1}));
        assert_eq!(span["kind"], 5);
    }

    #[test]
    fn kind_codes_follow_otlp() {
        assert_eq!(kind_code(SpanKind::Client), 3);
        assert_eq!(kind_code(SpanKind::Producer), 4);
    }
}
