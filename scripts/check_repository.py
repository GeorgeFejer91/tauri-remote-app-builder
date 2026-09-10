#!/usr/bin/env python3
"""Deterministic structural checks for the combined skill and reusable starter."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from urllib.parse import unquote


ROOT = Path(__file__).resolve().parents[1]
SKILL_NAME = "tauri-remote-app-builder"
SKILL_INVOCATION = f"${SKILL_NAME}"
IGNORED_DIRECTORIES = {".git", "__pycache__", "node_modules", "target"}
REQUIRED_PRODUCT_SURFACES = {"tauri", "localCli", "remoteCli", "browserCompanion"}
REQUIRED_OPERATION_KINDS = {"action", "query", "subscription", "transfer"}

# Inline links include ordinary links and images. Reference-style destinations
# are collected separately so every repository-authored Markdown target is
# checked without trying to fetch external URLs during an offline validation.
INLINE_MARKDOWN_LINK = re.compile(r"!?\[[^\]\n]*\]\(([^)\n]+)\)")
REFERENCE_MARKDOWN_LINK = re.compile(
    r"^\s{0,3}\[[^\]\n]+\]:\s*(<[^>\n]+>|\S+)", re.MULTILINE
)

REQUIRED_INTEGRATED_REFERENCES = [
    "references/00-system-contract.md",
    "references/01-architecture-and-authority.md",
    "references/02-protocol-and-state.md",
    "references/03-authentication-and-devices.md",
    "references/04-transport-and-reachability.md",
    "references/05-file-transfer.md",
    "references/06-tauri-and-rust-integration.md",
    "references/07-hosted-browser-companion.md",
    "references/08-lifecycle-and-supervision.md",
    "references/09-performance-and-operations.md",
    "references/10-qualification.md",
    "references/11-zuradio-lessons.md",
    "references/12-sources-and-provenance.md",
    "references/13-tauri-application-foundation.md",
    "references/cli-and-automation.md",
    "references/durable-jobs-and-media.md",
    "references/integrated-app-blueprint.md",
    "references/interface-quality.md",
    "references/pretext-text-layout.md",
    "references/uncodixfy-upstream.md",
    "references/zuradio-project-contract.md",
]

REQUIRED_TAURI_REFERENCE_NAMES = [
    "browser-companion-reliability.md",
    "coverage-map.md",
    "desktop-integration.md",
    "files-and-persistence.md",
    "frontend-frameworks.md",
    "latency-critical-systems.md",
    "marionette-remote-control.md",
    "mobile-development.md",
    "networking.md",
    "performance-and-production.md",
    "plugin-development.md",
    "project-workflow.md",
    "provenance.md",
    "python-migration-source-ledger.md",
    "python-to-rust-migration.md",
    "release-distribution.md",
    "rust-quality.md",
    "sidecars-and-processes.md",
    "source-ledger.md",
    "system-integrations.md",
    "tauri-architecture.md",
    "tauri-security.md",
    "testing-debugging.md",
    "verification.md",
    "vr-xr-development.md",
]

REQUIRED_FILES = [
    "SKILL.md",
    "agents/openai.yaml",
    "README.md",
    "LICENSE",
    "THIRD_PARTY_NOTICES.md",
    "licenses/TAURI-BROWSER-REMOTE-CONTROL-LICENSE",
    "licenses/TAURI-RUST-DEVELOPER-LICENSE",
    "licenses/PRETEXT-LICENSE",
    "licenses/UNCODIXFY-LICENSE",
    "licenses/ZURADIO-LICENSE",
    *REQUIRED_INTEGRATED_REFERENCES,
    *(f"references/tauri/{name}" for name in REQUIRED_TAURI_REFERENCE_NAMES),
    "assets/starter/LICENSE",
    "assets/starter/NOTICE.md",
    "assets/starter/DEPENDENCY-LICENSES.md",
    "assets/starter/README.md",
    "assets/starter/contracts/application-profile.schema.json",
    "assets/starter/contracts/application-profile.example.json",
    "assets/starter/contracts/surface-registry.example.json",
    "assets/starter/contracts/validate_application_profile.py",
    "assets/starter/contracts/wire-envelope.schema.json",
    "assets/starter/contracts/wire-envelope.example.json",
    "assets/starter/contracts/remembered-device-record.schema.json",
    "assets/starter/contracts/transfer-manifest.schema.json",
    "assets/starter/contracts/transfer-manifest.example.json",
    "assets/starter/rust/Cargo.toml",
    "assets/starter/rust/Cargo.lock",
    "assets/starter/rust/README.md",
    "assets/starter/browser/package.json",
    "assets/starter/browser/package-lock.json",
    "assets/starter/cli/Cargo.toml",
    "assets/starter/cli/Cargo.lock",
    "assets/starter/cli/LICENSE",
    "assets/starter/cli/NOTICE.md",
    "assets/starter/cli/README.md",
    "assets/starter/cli/src/lib.rs",
    "assets/starter/cli/src/main.rs",
    "assets/starter/cli/src/intent.rs",
    "assets/starter/cli/src/adapter.rs",
    "assets/starter/cli/src/output.rs",
    "assets/starter/cli/tests/cli_integration.rs",
    "assets/templates/VERIFICATION-GATE.md",
]


def repository_files(root: Path, suffix: str) -> list[Path]:
    """Return authored repository files while excluding generated dependency output."""

    return sorted(
        (
            path
            for path in root.rglob(f"*{suffix}")
            if not any(part in IGNORED_DIRECTORIES for part in path.relative_to(root).parts)
        ),
        key=lambda path: path.as_posix(),
    )


def markdown_destination(raw: str) -> str:
    """Extract a destination from an inline or reference-style Markdown link."""

    destination = raw.strip()
    if destination.startswith("<"):
        closing = destination.find(">")
        return destination[1:closing] if closing >= 0 else destination
    return destination.split(maxsplit=1)[0]


def local_markdown_targets(path: Path) -> list[Path]:
    text = path.read_text(encoding="utf-8")
    targets: list[Path] = []
    raw_targets = [
        *INLINE_MARKDOWN_LINK.findall(text),
        *REFERENCE_MARKDOWN_LINK.findall(text),
    ]
    for raw in raw_targets:
        target = markdown_destination(raw)
        lowered = target.lower()
        if not target or target.startswith("#") or lowered.startswith(
            ("http://", "https://", "mailto:")
        ):
            continue
        target = unquote(target.split("#", 1)[0].split("?", 1)[0])
        if target:
            targets.append((path.parent / target).resolve())
    return targets


def frontmatter(skill: str) -> tuple[str, str]:
    match = re.match(r"\A---\r?\n(.*?)\r?\n---\r?\n", skill, re.DOTALL)
    if not match:
        raise ValueError("SKILL.md is missing YAML frontmatter")
    name_match = re.search(r"^name:\s*(.+)$", match.group(1), re.MULTILINE)
    description_match = re.search(r"^description:\s*(.+)$", match.group(1), re.MULTILINE)
    if not name_match or not description_match:
        raise ValueError("SKILL.md frontmatter needs name and description")
    return name_match.group(1).strip(), description_match.group(1).strip()


def validate_profile(profile: object, registry: object | None = None) -> list[str]:
    """Validate the example's unified operation catalogue and four-surface parity."""

    errors: list[str] = []
    if not isinstance(profile, dict):
        return ["example profile must be a JSON object"]
    authority = profile.get("authority")
    adapters = profile.get("adapters")
    remote_control = profile.get("remoteControl")
    authentication = profile.get("authentication")
    transports = profile.get("transports")
    privacy = profile.get("privacy")
    if not isinstance(authority, dict) or authority.get("owner") != "rust":
        errors.append("example profile must make Rust authoritative")
    expected_adapters = {
        "tauri": "typed-product-ipc",
        "localCli": "authenticated-local-ipc-same-authority",
        "remoteCli": "same-public-protocol-as-browser",
        "browserCompanion": "static-untrusted-client",
    }
    if not isinstance(adapters, dict) or adapters != expected_adapters:
        errors.append("example profile must enable all four typed authority adapters")
    if not isinstance(remote_control, dict):
        errors.append("example profile must declare remoteControl")
        remote_control = {}
    if remote_control.get("policy") != "full-semantic-parity":
        errors.append("example profile must declare full-semantic-parity remote control")
    if remote_control.get("inventorySource") != "rust-domain-registry":
        errors.append("example profile inventory must originate from the Rust domain registry")
    if remote_control.get("productOperationInventoryComplete") is not True:
        errors.append("example profile must declare a complete product operation inventory")
    local_only_exceptions = remote_control.get("localOnlyExceptions")
    if not isinstance(local_only_exceptions, list):
        errors.append("example profile must declare localOnlyExceptions")
    elif local_only_exceptions:
        errors.append("example profile must demonstrate parity without local-only exceptions")

    scopes_value = profile.get("scopes")
    scopes = set(scopes_value) if isinstance(scopes_value, list) else set()
    operations = profile.get("operations")
    operation_ids: set[str] = set()
    kinds: set[str] = set()
    if not isinstance(operations, list) or not operations:
        errors.append("example profile must declare a unified product operation registry")
        operations = []
    for index, operation in enumerate(operations):
        if not isinstance(operation, dict):
            errors.append(f"example operation {index} must be a JSON object")
            continue
        operation_id = operation.get("id")
        if not isinstance(operation_id, str) or not operation_id:
            errors.append(f"example operation {index} must have an id")
            continue
        if operation_id in operation_ids:
            errors.append(f"example operation id is duplicated: {operation_id}")
        operation_ids.add(operation_id)
        kind = operation.get("kind")
        if kind not in REQUIRED_OPERATION_KINDS:
            errors.append(f"example operation {operation_id} has an invalid kind")
        else:
            kinds.add(kind)
        if operation.get("scope") not in scopes:
            errors.append(f"example operation {operation_id} references an undeclared scope")
        surfaces = operation.get("surfaces")
        if (
            not isinstance(surfaces, dict)
            or set(surfaces) != REQUIRED_PRODUCT_SURFACES
            or any(not isinstance(value, str) or not value for value in surfaces.values())
        ):
            errors.append(
                f"example operation {operation_id} must bind tauri, localCli, remoteCli, "
                "and browserCompanion"
            )
        for field in ("precondition", "idempotency", "resultSchema"):
            if not isinstance(operation.get(field), str) or not operation[field]:
                errors.append(f"example operation {operation_id} must declare {field}")
        if kind == "transfer":
            phases = operation.get("phases")
            phase_set = (
                {phase for phase in phases if isinstance(phase, str)}
                if isinstance(phases, list)
                else set()
            )
            if not operation.get("bulkProfile") or not {
                "declare",
                "stream",
                "finalize",
                "cancel",
            }.issubset(phase_set):
                errors.append(
                    f"example transfer {operation_id} must declare its bulk profile and transaction phases"
                )
    if kinds != REQUIRED_OPERATION_KINDS:
        errors.append("example registry must demonstrate actions, queries, subscriptions, and transfers")

    if registry is not None:
        if not isinstance(registry, dict) or not isinstance(registry.get("registries"), dict):
            errors.append("surface registry example must contain registries")
        else:
            registries = registry["registries"]
            if set(registries) != REQUIRED_PRODUCT_SURFACES:
                errors.append("surface registry example must contain exactly all four adapters")
            for surface in REQUIRED_PRODUCT_SURFACES:
                observed = registries.get(surface)
                if (
                    not isinstance(observed, list)
                    or any(not isinstance(value, str) for value in observed)
                    or len(observed) != len(set(observed))
                    or set(observed) != operation_ids
                ):
                    errors.append(f"surface registry {surface} must exactly match all product operations")

    remembered = authentication.get("rememberedBrowser") if isinstance(authentication, dict) else None
    maximum_ttl = remembered.get("maximumTtlSeconds") if isinstance(remembered, dict) else None
    if (
        not isinstance(maximum_ttl, int)
        or isinstance(maximum_ttl, bool)
        or maximum_ttl > 86_400
    ):
        errors.append("remembered-browser example must expire within 24 hours")
    if not isinstance(transports, dict) or transports.get("bulkLane") != "reliable-ordered-binary":
        errors.append("example profile must define a separate binary bulk lane")
    if not isinstance(privacy, dict) or any(
        privacy.get(field) is not False
        for field in ("staticHostContainsUserData", "nativePathsOnWire", "secretsInUrls")
    ):
        errors.append("example profile violates the static/privacy boundary")
    return errors


def validate_profile_schema(schema: object) -> list[str]:
    """Require a unified, closed product-operation and exception vocabulary."""

    if not isinstance(schema, dict):
        return ["application profile schema must be a JSON object"]
    properties = schema.get("properties")
    definitions = schema.get("$defs")
    root_required = schema.get("required")
    remote_control = properties.get("remoteControl") if isinstance(properties, dict) else None
    remote_required = remote_control.get("required") if isinstance(remote_control, dict) else None
    remote_properties = (
        remote_control.get("properties") if isinstance(remote_control, dict) else None
    )
    exceptions = (
        remote_properties.get("localOnlyExceptions")
        if isinstance(remote_properties, dict)
        else None
    )
    exception_items = exceptions.get("items") if isinstance(exceptions, dict) else None
    exception_required = (
        exception_items.get("required") if isinstance(exception_items, dict) else None
    )
    operation = definitions.get("operation") if isinstance(definitions, dict) else None
    operation_required = operation.get("required") if isinstance(operation, dict) else None
    operation_properties = operation.get("properties") if isinstance(operation, dict) else None
    kind = operation_properties.get("kind") if isinstance(operation_properties, dict) else None
    shared_surfaces = (
        definitions.get("surfaceBindings") if isinstance(definitions, dict) else None
    )
    shared_required = (
        shared_surfaces.get("required") if isinstance(shared_surfaces, dict) else None
    )
    local_surfaces = (
        definitions.get("localSurfaceBindings") if isinstance(definitions, dict) else None
    )

    errors: list[str] = []
    root_required_names = set(root_required) if isinstance(root_required, list) else set()
    if not {"remoteControl", "operations"}.issubset(root_required_names):
        errors.append("application profile schema must require remoteControl and operations")
    remote_required_names = set(remote_required) if isinstance(remote_required, list) else set()
    if not {
        "policy",
        "inventorySource",
        "productOperationInventoryComplete",
        "localOnlyExceptions",
    }.issubset(remote_required_names):
        errors.append("application profile schema must require a complete Rust-owned parity policy")
    policy = remote_properties.get("policy") if isinstance(remote_properties, dict) else None
    if not isinstance(policy, dict) or policy.get("const") != "full-semantic-parity":
        errors.append("application profile schema must require full-semantic-parity policy")
    exception_required_names = (
        set(exception_required) if isinstance(exception_required, list) else set()
    )
    if not {
        "operationId",
        "kind",
        "scope",
        "consequential",
        "category",
        "reason",
        "surfaces",
    }.issubset(exception_required_names):
        errors.append("application profile schema must fully describe local-only exceptions")
    operation_required_names = (
        set(operation_required) if isinstance(operation_required, list) else set()
    )
    if not {
        "id",
        "kind",
        "scope",
        "consequential",
        "precondition",
        "idempotency",
        "surfaces",
        "resultSchema",
    }.issubset(operation_required_names):
        errors.append("application profile schema must require complete operation metadata")
    kind_values = kind.get("enum") if isinstance(kind, dict) else None
    if not isinstance(kind_values, list) or set(kind_values) != REQUIRED_OPERATION_KINDS:
        errors.append("application profile schema must model actions, queries, subscriptions, and transfers")
    if not isinstance(shared_surfaces, dict) or shared_surfaces.get("additionalProperties") is not False:
        errors.append("shared surface bindings must reject unknown adapters")
    if not isinstance(shared_required, list) or set(shared_required) != REQUIRED_PRODUCT_SURFACES:
        errors.append("shared operations must require all four concrete adapter bindings")
    if not isinstance(local_surfaces, dict) or local_surfaces.get("additionalProperties") is not False:
        errors.append("local-only bindings must be a closed local adapter set")
    return errors


def validate_repository(root: Path = ROOT) -> list[str]:
    root = root.resolve()
    errors: list[str] = []
    for relative in REQUIRED_FILES:
        path = root / relative
        if not path.is_file():
            errors.append(f"missing required file: {relative}")
        elif path.stat().st_size == 0:
            errors.append(f"required file is empty: {relative}")

    skill_path = root / "SKILL.md"
    if skill_path.is_file():
        try:
            skill = skill_path.read_text(encoding="utf-8")
            name, description = frontmatter(skill)
            if name != SKILL_NAME:
                errors.append(f"unexpected skill name: expected {SKILL_NAME}")
            if not (80 <= len(description) <= 1024):
                errors.append("skill description is not sufficiently discriminating")
            for required in (
                "one Rust authority",
                "CLI",
                "browser companion",
                "PAKE",
                "bulk",
                "interface-quality",
                "Pretext",
                "qualification",
            ):
                if required.lower() not in skill.lower():
                    errors.append(f"SKILL.md is missing required concept: {required}")
        except (UnicodeDecodeError, ValueError) as error:
            errors.append(str(error))

    yaml_path = root / "agents" / "openai.yaml"
    if yaml_path.is_file():
        try:
            yaml = yaml_path.read_text(encoding="utf-8")
            if SKILL_INVOCATION not in yaml:
                errors.append(f"openai.yaml default prompt must name {SKILL_INVOCATION}")
            match = re.search(r'^\s*short_description:\s*"([^"]+)"', yaml, re.MULTILINE)
            if not match or not (25 <= len(match.group(1)) <= 64):
                errors.append("openai.yaml short_description must be 25-64 characters")
        except UnicodeDecodeError as error:
            errors.append(f"invalid UTF-8 agents/openai.yaml: {error}")

    parsed_json: dict[Path, object] = {}
    for path in repository_files(root, ".json"):
        try:
            parsed_json[path] = json.loads(path.read_text(encoding="utf-8"))
        except (json.JSONDecodeError, UnicodeDecodeError) as error:
            errors.append(f"invalid JSON {path.relative_to(root).as_posix()}: {error}")

    profile_path = root / "assets/starter/contracts/application-profile.example.json"
    surface_registry_path = root / "assets/starter/contracts/surface-registry.example.json"
    if profile_path in parsed_json:
        errors.extend(
            validate_profile(
                parsed_json[profile_path],
                parsed_json.get(surface_registry_path),
            )
        )
    profile_schema_path = root / "assets/starter/contracts/application-profile.schema.json"
    if profile_schema_path in parsed_json:
        errors.extend(validate_profile_schema(parsed_json[profile_schema_path]))

    for path in repository_files(root, ".md"):
        try:
            text = path.read_text(encoding="utf-8")
            if "[TODO" in text or "TODO:" in text:
                errors.append(f"unresolved TODO marker in {path.relative_to(root).as_posix()}")
            for target in local_markdown_targets(path):
                try:
                    target.relative_to(root)
                except ValueError:
                    errors.append(
                        "local link escapes repository in "
                        f"{path.relative_to(root).as_posix()}: {target}"
                    )
                    continue
                if not target.exists():
                    errors.append(
                        "broken local link in "
                        f"{path.relative_to(root).as_posix()}: "
                        f"{target.relative_to(root).as_posix()}"
                    )
        except UnicodeDecodeError as error:
            errors.append(
                f"invalid UTF-8 Markdown {path.relative_to(root).as_posix()}: {error}"
            )

    return errors


def main() -> int:
    errors = validate_repository()
    if errors:
        print("Repository validation failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print("Repository validation passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
