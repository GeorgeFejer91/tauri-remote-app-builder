# Remote access threat model

Date:
Version/commit:

## Assets

List product state, user files/media, device capabilities, password/verifier, device records, grants, signing/update keys, logs, and availability.

## Principals

List local user, observer/controller/uploader roles, remembered browsers, native services, static host, signaling/TURN operator, sidecars, and update operator.

## Entry points and trust boundaries

- hosted companion origin and supply chain;
- discovery/signaling/STUN/TURN;
- WebRTC/WSS/WebTransport frames;
- Tauri IPC and remote-origin capabilities;
- loopback HTTP/WebSocket/bootstrap;
- filesystem transfer staging/download;
- sidecars/process supervision;
- CI/signing/update/distribution;
- logs/support diagnostics.

## Abuse cases and mitigations

For each case record prevention, detection, recovery, test, and residual risk:

- password offline/online guessing;
- transcript replay/reflection;
- stolen browser credential or same-origin script compromise;
- scope/role escalation and stale grants;
- duplicate/reordered command side effects;
- hostile/oversized frames and queue exhaustion;
- arbitrary path, symlink, URL, shell, SQL, or command injection;
- transfer corruption, disk exhaustion, abandoned staging;
- malicious signaling listing/peer replacement/relay metadata;
- loopback request from another local process or hostile web origin;
- host/authority restart races and stale callbacks;
- beacon availability causing unintended activation;
- sidecar compromise/malformed output/license/update failure;
- stale public companion or compromised release pipeline.

## Data-flow privacy table

| Data class | Owner | Allowed recipient/scope | Transport | Retention | Logs | Delete/revoke |
|---|---|---|---|---|---|---|

## Assumptions

State OS/session trust, browser-origin trust, physical access, signaling/relay behavior, and what a compromised same-origin script can do.

## Verification

Link each mitigation to a unit, contract, browser, network, installed, or operational test. Mark missing evidence explicitly.
