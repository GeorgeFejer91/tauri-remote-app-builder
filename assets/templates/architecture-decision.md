# Remote access architecture decision

Status: proposed
Date:
Owners:

## Product outcome

Describe what a remote browser may observe and control, what it must never do, and the supported phone/computer/browser platforms.

## Authority

- Rust authority process/module:
- Domain action enum/location:
- Revision owner and persistence:
- Local UI, CLI, Tauri, and remote adapters:
- Multi-controller/arbitration policy:

## Availability

- Mode: manual session / app-lifetime beacon / supervised login service
- Explicit user enablement:
- Deliberate-off behavior:
- Remote availability versus product activation:

## Topology

- Static companion origin/deployment:
- Browser host adapter per native platform:
- Local native boundary:
- Signaling/STUN/TURN/WSS owner:
- Universal and optional fast paths:
- Direct/relay policy and route evidence:

## Protocol

- Name/version:
- Roles/scopes/actions:
- Public state and revision rules:
- Control/state/bulk/media lanes:
- Dedupe and retry behavior:
- Compatibility window:

## Authentication and authorization

- Human-password PAKE and implementation:
- High-entropy invitation profile, if any:
- 24-hour device credential form:
- Grant fields and expiry:
- Revoke/password-change behavior:
- Attempt limits and recovery:

## File/media boundary

- Allowed upload/download objects and limits:
- Staging/finalization/digest/resume policy:
- Static-host privacy boundary:
- Optional post-processing/sidecars/licenses:

## Performance budgets

- Cold/trusted connection:
- Idle/saturated control p95/p99:
- First snapshot/file publication:
- Bulk throughput/resources:
- Recovery:

## Qualification matrix

List browsers, system WebViews, native platforms, physical devices, LAN/NAT/TURN/restrictive networks, packaged/install/public paths, signing/update, soak/endurance, and owners for each.

## Rejected alternatives

Record why generic remote input, direct remote Tauri capability, public loopback exposure, bearer-only trust, or another transport was rejected/accepted.

## Residual risk

State unresolved threats and intentionally unqualified claims.
