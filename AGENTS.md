# Repository Instructions

> Mandatory first read: [for-ai/README.md](./for-ai/README.md). It is the YAGNI control-plane router; then follow the project-specific rules below.

Read `SKILL.md` and every reference routed for the requested change before editing.

Preserve the central boundary: one Rust authority, a closed typed action set, Tauri/UI/CLI/remote adapters over the same service, explicit authentication and authorization, bounded resources, and evidence-based claims. Remote-ready does not authorize opening listeners, changing firewall/autostart settings, creating public services, or deploying.

Keep `SKILL.md` a useful router. Put conditional detail in focused references and avoid duplicating the same rule across multiple files. Preserve source provenance and third-party notices whenever copied or adapted material changes.

Changes to contracts, CLI parsing, starter code, authentication, transfer framing, scaffold behavior, or qualification requirements need deterministic tests and a realistic forward-test assessment. Run every applicable command in the README validation block. Repository checks do not qualify a generated application as production-ready.
