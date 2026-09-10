# Rust authority starter

This crate demonstrates one transport-neutral Rust authority, scoped grants, monotonic revisions, expected-revision checks, and command deduplication. It deliberately contains no network transport, password protocol, persistence, Tauri IPC, or generic command dispatcher.

The in-memory dedupe table stores both applied outcomes and deterministic final rejections after authentication. Retryable conditions such as temporary unavailability or capacity pressure must not be recorded as final. Generated applications must classify every rejection and return the same final outcome for the same authority generation, principal, command ID, and request fingerprint.

The table is intentionally bounded and volatile. If a generated app commits jobs, files, payments, messages, or other effects that survive restart, persist the final outcome or a command-ID tombstone in the same durable transaction as the side effect. Do not claim restart-safe idempotency from this starter alone.

Replace the example `AppAction` and `AppState` with the application's closed domain types. Generate or test the operation and adapter registries used by `contracts/validate_application_profile.py`; browser, Tauri, local CLI, and remote CLI bindings must resolve to these same Rust operations.

Run:

```bash
cargo fmt --all --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
```
