# Tauri Remote App Builder

A combined Codex skill for creating polished Rust/Tauri applications with one native authority shared by the desktop interface, a first-class CLI, and a paired browser or phone companion with full semantic product-control parity.

It integrates five sources into one routed workflow:

- [Tauri Browser Remote Control](https://github.com/GeorgeFejer91/tauri-browser-remote-control) for typed remote actions, authentication, state synchronization, transport, transfer, lifecycle, and qualification;
- [Tauri Rust Developer Skill](https://github.com/GeorgeFejer91/tauri-rust-developer-skill) for general Tauri/Rust architecture, security, plugins, sidecars, persistence, mobile/desktop integration, migration, performance, testing, and release work;
- [Zuradio Builder](https://github.com/GeorgeFejer91/zuradio/tree/main/skills/zuradio-builder) for production-derived lessons around local authority, CLI parity, WebRTC media/control, uploads, remembered browsers, supervision, and installed/public validation;
- [Uncodixfy](https://github.com/cyxzdev/uncodixfy) for an anti-template UI critique, adapted so existing design systems, accessibility, semantics, and platform conventions remain controlling;
- [Pretext](https://github.com/chenglou/pretext) for predictive multiline text geometry, bounded-text policies, virtualization, responsive reflow, and testable overflow prevention within its documented support profile.

## What it builds

```text
Tauri desktop UI ----+
local typed CLI -----+----> one Rust authority ----> persistence / OS / workers
remote typed CLI ----+
browser companion ---+      authenticated, scoped, revisioned adapters
```

The skill treats remote access as application-semantic control, not remote desktop. Every ordinary product action, query, subscription/progress stream, and transactional transfer must be mapped to the installed UI, CLI, remote CLI, and browser companion; any local-only trust, consent, physical-presence, OS-permission, or safety exception must be explicit and tested. It excludes arbitrary shell, input injection, unrestricted filesystem access, generic Tauri command forwarding, and silent network/background enablement.

For new projects it expects:

- a pure Rust domain/service boundary;
- thin Tauri IPC and platform adapters;
- a typed CLI with stable JSON and human output;
- a Rust-derived, machine-checked operation registry with full browser/CLI parity;
- explicit activation, authentication, authorization, revocation, and state recovery;
- a static companion that contains no user data or native authority in the local-first profile;
- product-specific, accessible UI without generic generated-dashboard styling;
- allow-listed typography and explicit overflow outcomes for text regions that use Pretext prediction;
- proportional source, browser, packaged-runtime, device, and release evidence.

## Install and invoke

Install from GitHub with a skill-compatible installer or copy this repository into your Codex skills directory. The root [SKILL.md](SKILL.md) is the only skill entry point.

```text
$tauri-remote-app-builder
```

Example:

```text
Use $tauri-remote-app-builder to create a Tauri v2 desktop app with a shared Rust core, an appctl CLI, and an opt-in phone companion. Keep the interface product-specific and qualify the real browser-to-Rust path.
```

## Repository map

```text
SKILL.md                       compact operating contract and router
agents/openai.yaml             Codex display metadata
references/                    integrated and focused guidance
references/tauri/              complete general Tauri/Rust reference library
assets/starter/                contracts, Rust/browser/CLI starter boundaries
assets/templates/              architecture, threat-model, and gate templates
scripts/scaffold_remote_control.py
scripts/check_repository.py
tests/                         structural, scaffold, and forward-test material
```

The starter deliberately does not invent a password protocol, production endpoint, signaling account, generic local IPC transport, or mandatory frontend dependency. A generated application must select maintained implementations, map its exact operations, generate adapter-registry evidence, and qualify the installed path. When Pretext is selected, [references/pretext-text-layout.md](references/pretext-text-layout.md) defines the strict supported typography and overflow-verification contract.

## Validate this skill

```bash
python3 scripts/check_repository.py
python3 -m unittest discover -s tests -v
cargo fmt --all --manifest-path assets/starter/rust/Cargo.toml -- --check
cargo clippy --locked --all-targets --manifest-path assets/starter/rust/Cargo.toml -- -D warnings
cargo test --locked --manifest-path assets/starter/rust/Cargo.toml
cargo fmt --all --manifest-path assets/starter/cli/Cargo.toml -- --check
cargo clippy --locked --all-targets --manifest-path assets/starter/cli/Cargo.toml -- -D warnings
cargo test --locked --manifest-path assets/starter/cli/Cargo.toml
npm --prefix assets/starter/browser ci --ignore-scripts
npm --prefix assets/starter/browser test
```

These checks validate the reusable skill and starter, not an application's production readiness. See [references/10-qualification.md](references/10-qualification.md) for application-level evidence.

## Provenance

Exact immutable source revisions and destination mappings are recorded in [references/12-sources-and-provenance.md](references/12-sources-and-provenance.md). See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for required notices.

MIT licensed.
