# Remote-enabled application instructions

Read the project architecture decision, threat model, and `VERIFICATION-GATE.md` before changing remote access, Tauri IPC/capabilities, authority state, authentication, transport, file/media transfer, browser lifecycle, supervision, packaging, or deployment.

Preserve these invariants:

- Rust is the sole authority.
- Remote messages map only to closed typed product actions.
- Human passwords use the project's reviewed PAKE; remembered-browser access is revocable and expires within 24 hours.
- Control, replaceable state, bulk bytes, and media have explicit independent semantics and bounded resources.
- Static hosting contains no user data or secrets.
- Remote availability never activates the controlled product function.
- No completion claim is valid until the mandatory real browser/native gate passes.

Do not weaken a latency, throughput, security, browser, device, installed, or network gate merely to obtain a passing result. Report unavailable gates as open and narrow the claim.
