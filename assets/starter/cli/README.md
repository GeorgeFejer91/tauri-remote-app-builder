# Typed CLI/admin adapter starter

This crate is a reusable command-line edge for the application's **running
Rust authority**. It parses a closed command grammar, calls one typed
`AdminAdapter` method, and emits revisioned human or JSON output. It owns no
mutable application state, credentials, device registry, server lifecycle, or
revision counter.

Replace `UnwiredAdapter` in `src/main.rs` with an adapter to the same in-process
service or authenticated local IPC service used by the Tauri shell. Do not
build a parallel CLI database or duplicate backend. Copy `LICENSE` and
`NOTICE.md` with the scaffolded crate.

## Fixed commands

The only accepted verbs are:

```text
appctl [--output human|json] remote start --exposure local-network|configured-relay --allow-new-devices true|false
appctl [--output human|json] remote status
appctl [--output human|json] remote stop --force true|false
appctl [--output human|json] devices list --filter current|revoked|all
appctl [--output human|json] devices revoke --device-id ID --reason user-requested|lost|compromised
appctl [--output human|json] doctor --area all|authority|transport|browser
appctl [--output human|json] product status
appctl [--output human|json] product set-level --value 0..100 --expected-revision REVISION --command-id ID --timeout-ms 100..120000
appctl [--output human|json] product set-active --active true|false --expected-revision REVISION --command-id ID --timeout-ms 100..120000
```

Options may be reordered within a command, but are required exactly once.
Booleans accept only `true` or `false`; enums accept only the values shown.
Device IDs are bounded ASCII identifiers. Unknown options, duplicate options,
extra positional arguments, malformed values, and generic forwarding commands
such as `invoke`, `exec`, or `--payload` are rejected before adapter dispatch.
Command IDs are opaque 8–128 byte ASCII identifiers using letters, digits,
`_`, `.`, `:`, or `-`. Consequential product mutations require the caller to
supply one stable command ID and an explicit timeout from 100 through 120,000
milliseconds.

`configured-relay` selects relay settings already reviewed and stored by the
application. The CLI intentionally does not accept arbitrary endpoint URLs,
bind addresses, shell strings, Tauri command names, or JSON payloads.

The `product` group is the **complete** semantic action surface for the starter
Rust authority: `set-level` maps to `AppAction::SetLevel`, while the two values
of `set-active` map to `AppAction::Activate` and `AppAction::Deactivate`.
`product status` returns the same authoritative snapshot used by both control
edges. These are not an ad hoc subset of example operations.

When generating a real application, inventory its Rust semantic action enum and
replace the starter product variants, browser protocol variants, and typed CLI
adapter methods together. Every ordinary product action must have a one-to-one
paired-browser and CLI mapping with the same validation, scope, expected-
revision, applied result, and authoritative snapshot semantics. Never use a
generic command as an escape hatch for incomplete coverage.

`CommandName::availability()` makes exclusions explicit:

| Commands | Availability | Reason |
| --- | --- | --- |
| `product status`, `product set-level`, `product set-active` | `paired_browser_and_cli` | Shared semantic product control |
| `remote *`, `devices *`, `doctor` | `local_admin_only` | Local lifecycle, trust administration, and diagnostics |

A generated app may add another local-only exclusion only when the action is
not needed to operate the product remotely and its rationale is documented.
Keep revision guards on all mutations.

`remote status` reports one explicit lifecycle/readiness value: `stopped`,
`discovering`, `authenticating`, `control_ready`, `media_ready`, `degraded`,
`reconnecting`, `revoked`, or `stopping`. Do not collapse transport-open into
control readiness or hide authentication, recovery, or revocation states.

## Output contract

Both formats use schema version `appctl.v1`. A JSON success has a stable top
level shape:

```json
{
  "schemaVersion": "appctl.v1",
  "ok": true,
  "command": "product.set_level",
  "commandId": "cli_level_0001",
  "authorityGeneration": "authority_demo_01",
  "revision": 8,
  "data": {
    "type": "product_state",
    "value": { "active": true, "level": 37 }
  }
}
```

Failures go to stderr and contain only stable public error codes and messages.
`AdapterError` deliberately cannot carry internal error text, paths, tokens,
peer data, stack traces, or transport responses into CLI output.

| Exit | Meaning |
| ---: | --- |
| `0` | Applied operation or completed read |
| `2` | CLI grammar or value error |
| `3` | Authentication missing or failed |
| `4` | Authorization or scope denied |
| `5` | Conflict, stale revision, or rejected state precondition |
| `6` | Authority unavailable, disconnected, busy, or transport failure |
| `7` | Timed out before the authoritative outcome was known |
| `8` | Internal failure with only safe diagnostics exposed |

Human output is a deterministic `key=value` form with explicit booleans and
quoted dynamic identifiers. Successful output includes both the authority
generation and revision returned by the running service.

A `timed_out_unknown_outcome` failure is not a rejected application action.
Reconcile state or retry the logically identical request with the same command
ID; never mint a fresh ID merely because an acknowledgement was lost. Success
and adapter-failure envelopes echo the mutation's command ID so automation can
correlate retries without parsing prose.

## Wiring pattern

Implement all `AdminAdapter` methods on a narrow application adapter. Each
method should authenticate to or call the existing authority, await its
committed result, and return the authority's generation and revision in
`Revisioned<T>`. Map private failures to the nearest `AdapterErrorCode` at that
boundary and keep detailed diagnostics in access-controlled application logs.

Never report a locally incremented CLI revision. Never treat successful
transport delivery as committed authority state. A `product set-*` adapter
must forward `expected_revision` to the authority and return `Conflict` when it
is stale. It must also forward `command_id`, enforce `timeout_ms` at the real
request boundary, return the same command ID with an acknowledged result, and
return `TimedOutUnknownOutcome` when the timeout expires without a known
authority decision.

## Validation

```bash
cargo fmt --manifest-path Cargo.toml --check
cargo clippy --manifest-path Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path Cargo.toml
```

The integration tests inject a same-service mock, exercise every fixed verb,
assert revisioned and command-correlated results, verify stable human/JSON
shapes and exit codes, exercise every readiness state, and prove malformed or
generic forwarding input never reaches the adapter. A focused parity test also
sends the parsed shared `product set-level` intent through
`remote_control_core_starter::Authority`, proving the CLI result carries the
actual reducer's committed command ID, revision, and state.
