<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<p align="center">
  <img src="https://cloudcdn.pro/rlg/v1/logos/rlg.svg" alt="RLG logo" width="128" />
</p>

<h1 align="center">rlg-wasm</h1>

<p align="center">
  WebAssembly bindings for <code>rlg</code>. Structured logging in
  browsers, Deno, Cloudflare Workers, Bun.
</p>

<p align="center">
  <a href="https://github.com/sebastienrousseau/rlg/actions"><img src="https://img.shields.io/github/actions/workflow/status/sebastienrousseau/rlg/ci.yml?style=for-the-badge&logo=github" alt="Build" /></a>
  <a href="https://crates.io/crates/rlg-wasm"><img src="https://img.shields.io/crates/v/rlg-wasm.svg?style=for-the-badge&color=fc8d62&logo=rust" alt="Crates.io" /></a>
  <a href="https://docs.rs/rlg-wasm"><img src="https://img.shields.io/badge/docs.rs-rlg--wasm-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="https://lib.rs/crates/rlg-wasm"><img src="https://img.shields.io/badge/lib.rs-rlg--wasm-orange.svg?style=for-the-badge" alt="lib.rs" /></a>
  <a href="https://scorecard.dev/viewer/?uri=github.com/sebastienrousseau/rlg"><img src="https://img.shields.io/ossf-scorecard/github.com/sebastienrousseau/rlg?style=for-the-badge&label=OpenSSF%20Scorecard&logo=openssf" alt="OpenSSF Scorecard" /></a>
</p>

---

## Install

```toml
[dependencies]
rlg-wasm = "0.0.11"
```

Requires Rust **1.88.0** or newer (edition 2024). For the wasm32 target, install `wasm-pack` as shown below.

## Build a WASM bundle

```bash
cargo install wasm-pack
wasm-pack build crates/rlg-wasm --target web --release
```

The `pkg/` output contains the JS shims, TypeScript types, and the
`.wasm` artifact.

## Browser / Deno / Bun usage

```js
import init, { RlgWasm } from "./pkg/rlg_wasm.js";

await init();

const rlg = new RlgWasm("worker", "JSON");
rlg.info("worker booted", JSON.stringify({ region: "eu-west-1" }));
rlg.warn("rate limited", null);
rlg.error("db timeout", JSON.stringify({ db: "primary", elapsed_ms: 5012 }));
```

Records render in the chosen `LogFormat` and dispatch to the host's
`console.log` / `console.warn` / `console.error` based on the level.

## Cloudflare Workers

```toml
# wrangler.toml
name = "my-worker"
main = "src/index.js"
compatibility_date = "2026-05-30"

[build]
command = "wasm-pack build --target web"
```

```js
// src/index.js
import init, { RlgWasm } from "../pkg/rlg_wasm.js";
import wasm from "../pkg/rlg_wasm_bg.wasm";

let rlg;

export default {
    async fetch(request, env, ctx) {
        if (!rlg) {
            await init(wasm);
            rlg = new RlgWasm("my-worker", "JSON");
        }
        rlg.info("request received", JSON.stringify({
            url: request.url,
            method: request.method,
        }));
        return new Response("ok");
    },
};
```

## Supported formats

All 14 `LogFormat` variants: `CLF`, `CEF`, `ELF`, `W3C`,
`ApacheAccessLog`, `Log4jXML`, `JSON`, `GELF`, `Logstash`, `NDJSON`,
`MCP`, `OTLP`, `Logfmt`, `ECS`. Pass the variant name as the second
constructor argument.

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT), at your option.
