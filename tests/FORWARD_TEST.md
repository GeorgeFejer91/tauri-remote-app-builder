# Forward test: remote photo presenter

Date: 2026-09-02

## Prompt

Design the remote foundation for a new Tauri v2 photo-presentation app. A static phone/computer companion must observe status, start/stop, move next/previous, and change transition speed; upload multi-gigabyte photo batches; remember an approved browser for no more than 24 hours; remain discoverable through an explicitly enabled user-login service; work through ordinary NAT with direct/relay status; and keep control p95 below 200 ms while transfer is saturated.

## Skill routing exercised

- `13-tauri-application-foundation.md` for new-app setup and Rust/Tauri boundaries;
- `00`, `01`, and `02` for one authority, typed actions, revisions, dedupe, snapshots, timeline and local arbitration;
- `03` for OPAQUE/SPAKE selection, scoped grants, proof-of-possession browser key, expiry and revoke;
- `04` for WebRTC plus TURN, separate DataChannels, route evidence, WSS alternative and WebTransport boundary;
- `05` for manifest/staging/offset/digest/incremental batch publication and true resumption requirements;
- `07` and `08` for static hosting, phone lifecycle, explicit service consent, readiness and recovery without activation;
- `09` and `10` for the saturated p95 budget, resource ceilings, physical/browser/network, installed/public and supervision gates.

## Resulting design

- Rust owns slideshow state, revision, grants, command outcomes, selected stable asset IDs, transfer staging, and persistence.
- Local Tauri UI, CLI, and remote adapter share `Start`, `Stop`, `Next`, `Previous`, and bounded `SetTransitionMs` domain actions. No remote paths, generic Tauri command names, keyboard/mouse injection, or URL fetches are exposed.
- A static HTTPS companion discovers the installed host through signaling. WebRTC uses reliable ordered control, newest-only presentation state, and a reliable ordered binary bulk channel; TURN is provisioned and the selected route is displayed as direct/relay/unknown.
- Human login uses a maintained OPAQUE or SPAKE2+ implementation selected after ecosystem verification. Successful pairing can enroll a non-extractable WebCrypto signing key; Rust stores the public key/scopes and hard expiry no later than 24 hours. Password change/revoke/authority rotation invalidates device records and grants.
- The explicitly enabled login service supervises only the host adapter. Becoming discoverable never starts a slideshow. Exact-process replacement and deliberate-off behavior are mandatory tests.
- Uploads use private staging, declared IDs/sizes, exact offsets, negotiated-size binary frames, bounded acknowledgements/buffers, authoritative incremental SHA-256, atomic finalization, and per-file publication. Resumption is not claimed until persisted range/restart tests exist.
- Completion requires the public companion to apply a native action against the installed app, p95 control below 200 ms during a large saturated upload, direct and forced-relay cases, browser/physical phone lifecycle, process recovery, memory/queue/disk ceilings, and a 1 GiB soak if the multi-gigabyte promise remains.

## Safety decisions

The skill correctly refused to invent a PAKE crate, production bundle identity, signaling/TURN operator, signing identity, or background enablement on the user's behalf. Those choices depend on the actual project and require current dependency/platform review or explicit authority.

## Scaffold portability evidence

`scripts/scaffold_remote_control.py` copied the final starter into a fresh temporary `photo-presenter-remote-final` directory without build artifacts. The copied Rust crate passed 8 tests on Rust 1.74.0 with its lockfile; the copied browser package passed 6 Node 22-compatible tests. A second copy to the same destination is covered by the repository tests and fails closed rather than overwriting.

## Outcome

Pass. The skill produced a complete implementable architecture, preserved security/consent boundaries, exposed decisions that cannot be safely invented, and routed every promised property to a concrete qualification gate.
