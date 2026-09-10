# Starter dependency license inventory

Checked from both committed Cargo lockfiles on 2026-09-10.

The Rust authority starter directly uses `serde`, `serde_json`, and `sha2`.
The CLI starter directly uses `serde` and `serde_json`, and uses the authority
starter as a development dependency for its real-reducer integration test.
Those crates and their resolved transitive dependencies declare permissive
license expressions: MIT, Apache-2.0, `MIT OR Apache-2.0`, `Unlicense OR MIT`,
and, for `unicode-ident`, `(MIT OR Apache-2.0) AND Unicode-3.0`.

The browser starter has no third-party runtime or development packages in its
lockfile; it uses Node's built-in test runner.

This source inventory is not a binary-distribution SBOM. Generated applications
must regenerate a license inventory from their final lockfiles and retain the
license texts, notices, source offers, attribution, or other obligations
required by every dependency and bundled asset they actually ship.
