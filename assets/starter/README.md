# Remote-control starter

These files provide a transport-neutral boundary for a remote-enabled Rust/Tauri application. Copy them with `scripts/scaffold_remote_control.py`, then replace the example operation/state profile with product semantics.

The starter includes:

- JSON application-profile, four-surface registry, wire-envelope, remembered-device, and transfer schemas;
- a project-side parity validator for actions, queries, subscriptions/progress, and transactional transfers;
- a pure Rust authority with grant checks, revisions, expected-revision handling, and fingerprinted command deduplication;
- a typed `appctl` adapter with fixed remote/device/doctor verbs, complete product-action parity hooks, stable human/JSON output, and fail-closed executable behavior until wired to the running authority;
- strict browser protocol validation and binary-frame framing;
- a Playwright scenario outline for the real browser/native gate.

It intentionally does **not** include:

- a hand-written PAKE or remembered-device cryptography;
- signaling, STUN, TURN, VDO.Ninja credentials, or production endpoints;
- production local IPC between `appctl` and the chosen authority owner;
- arbitrary Tauri commands, filesystem paths, shell access, or generic remote input;
- a claim that copied code is production-ready.

Replace the example product operations with the application's Rust-derived action/query/subscription/transfer registry. Generate each surface list from its typed adapter, then run `python contracts/validate_application_profile.py`; a hand-edited list is not evidence of code coverage. Integrate a maintained implementation of OPAQUE, SPAKE2+, or SPAKE2 for human-password authentication, bind its authenticated result to the Rust grant API, connect the CLI to the same authority through reviewed local IPC, and qualify the complete installed path.

Retain `LICENSE`, `NOTICE.md`, and `DEPENDENCY-LICENSES.md` with copied starter code.

Run the standalone checks:

```bash
python contracts/validate_application_profile.py
cargo test --manifest-path rust/Cargo.toml
cargo test --manifest-path cli/Cargo.toml
npm --prefix browser test
```
