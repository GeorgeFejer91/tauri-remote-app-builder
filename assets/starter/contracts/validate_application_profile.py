#!/usr/bin/env python3
"""Fail-closed parity checks for a generated application's operation registries."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path, PurePosixPath, PureWindowsPath
from typing import Any


SURFACES = ("tauri", "localCli", "remoteCli", "browserCompanion")
LOCAL_SURFACES = ("tauri", "localCli")
KINDS = {"action", "query", "subscription", "transfer"}
EXCEPTION_CATEGORIES = {
    "trust",
    "consent",
    "physical-presence",
    "os-permission",
    "safety",
}
TOKEN = re.compile(r"^[A-Za-z0-9][A-Za-z0-9_.:-]{0,127}$")


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def _valid_binding(value: object) -> bool:
    return isinstance(value, str) and 0 < len(value) <= 256


def validate(
    profile: object,
    observed: object,
    base_directory: Path | None = None,
) -> list[str]:
    errors: list[str] = []
    if not isinstance(profile, dict):
        return ["profile must be a JSON object"]
    if not isinstance(observed, dict):
        return ["observed surface registry must be a JSON object"]

    scopes_value = profile.get("scopes")
    scopes = set(scopes_value) if isinstance(scopes_value, list) else set()
    if not scopes or any(not isinstance(scope, str) for scope in scopes):
        errors.append("profile must declare a non-empty unique scope list")

    schemas_value = profile.get("schemas")
    schemas = schemas_value if isinstance(schemas_value, dict) else {}
    if not schemas:
        errors.append("profile must declare closed product-specific schema documents")
    for schema_id, reference in schemas.items():
        if not isinstance(schema_id, str) or not TOKEN.fullmatch(schema_id):
            errors.append("schema IDs must be bounded tokens")
            continue
        if not isinstance(reference, str) or not reference:
            errors.append(f"schema {schema_id} must name a document")
            continue
        document = reference.split("#", 1)[0]
        posix = PurePosixPath(document)
        windows = PureWindowsPath(document)
        if (
            not document.endswith(".json")
            or posix.is_absolute()
            or windows.is_absolute()
            or bool(windows.drive)
            or ".." in posix.parts
            or "\\" in document
        ):
            errors.append(f"schema {schema_id} must use a safe relative JSON reference")
        elif base_directory is not None and not (base_directory / document).is_file():
            errors.append(f"schema {schema_id} document does not exist: {document}")

    remote = profile.get("remoteControl")
    if not isinstance(remote, dict):
        errors.append("remoteControl must be an object")
        remote = {}
    if remote.get("policy") != "full-semantic-parity":
        errors.append("remoteControl.policy must be full-semantic-parity")
    if remote.get("inventorySource") != "rust-domain-registry":
        errors.append("the operation inventory must originate from the Rust domain registry")
    if remote.get("productOperationInventoryComplete") is not True:
        errors.append("productOperationInventoryComplete must be true")

    operations = profile.get("operations")
    if not isinstance(operations, list) or not operations:
        errors.append("operations must be a non-empty array")
        operations = []

    expected: dict[str, set[str]] = {surface: set() for surface in SURFACES}
    known_ids: set[str] = set()
    for index, operation in enumerate(operations):
        label = f"operations[{index}]"
        if not isinstance(operation, dict):
            errors.append(f"{label} must be an object")
            continue
        operation_id = operation.get("id")
        if not isinstance(operation_id, str) or not TOKEN.fullmatch(operation_id):
            errors.append(f"{label}.id must be a bounded token")
            continue
        if operation_id in known_ids:
            errors.append(f"duplicate operation id: {operation_id}")
        known_ids.add(operation_id)
        if operation.get("kind") not in KINDS:
            errors.append(f"{operation_id} has an invalid operation kind")
        if operation.get("scope") not in scopes:
            errors.append(f"{operation_id} references an undeclared scope")
        if not isinstance(operation.get("consequential"), bool):
            errors.append(f"{operation_id} must declare consequential as a boolean")
        for field in ("precondition", "idempotency", "resultSchema"):
            if not isinstance(operation.get(field), str) or not operation[field].strip():
                errors.append(f"{operation_id} must declare {field}")
        for field in ("requestSchema", "resultSchema"):
            schema_id = operation.get(field)
            if schema_id is not None and schema_id not in schemas:
                errors.append(f"{operation_id}.{field} references an undeclared schema")
        surfaces = operation.get("surfaces")
        if not isinstance(surfaces, dict) or set(surfaces) != set(SURFACES):
            errors.append(f"{operation_id} must bind exactly all four product surfaces")
        else:
            for surface in SURFACES:
                if not _valid_binding(surfaces[surface]):
                    errors.append(f"{operation_id}.{surface} must be a bounded explicit binding")
                expected[surface].add(operation_id)
        if operation.get("kind") == "action" and not operation.get("requestSchema"):
            errors.append(f"action {operation_id} must declare requestSchema")
        if operation.get("kind") == "transfer":
            if not operation.get("bulkProfile"):
                errors.append(f"transfer {operation_id} must declare bulkProfile")
            phases = operation.get("phases")
            required_phases = {"declare", "stream", "finalize", "cancel"}
            phase_set = (
                {phase for phase in phases if isinstance(phase, str)}
                if isinstance(phases, list)
                else set()
            )
            if not required_phases.issubset(phase_set):
                errors.append(
                    f"transfer {operation_id} must declare declare/stream/finalize/cancel phases"
                )

    exceptions = remote.get("localOnlyExceptions")
    if not isinstance(exceptions, list):
        errors.append("localOnlyExceptions must be an array")
        exceptions = []
    exception_ids: set[str] = set()
    for index, exception in enumerate(exceptions):
        label = f"localOnlyExceptions[{index}]"
        if not isinstance(exception, dict):
            errors.append(f"{label} must be an object")
            continue
        operation_id = exception.get("operationId")
        if not isinstance(operation_id, str) or not TOKEN.fullmatch(operation_id):
            errors.append(f"{label}.operationId must be a bounded token")
            continue
        if operation_id in known_ids:
            errors.append(f"{operation_id} cannot be both shared and local-only")
        if operation_id in exception_ids:
            errors.append(f"duplicate local-only operation id: {operation_id}")
        exception_ids.add(operation_id)
        if exception.get("kind") not in KINDS:
            errors.append(f"{operation_id} has an invalid local-only operation kind")
        if exception.get("scope") not in scopes:
            errors.append(f"{operation_id} references an undeclared scope")
        if not isinstance(exception.get("consequential"), bool):
            errors.append(f"{operation_id} must declare consequential as a boolean")
        if exception.get("category") not in EXCEPTION_CATEGORIES:
            errors.append(f"{operation_id} has an invalid local-only category")
        reason = exception.get("reason")
        if not isinstance(reason, str) or len(reason.strip()) < 20:
            errors.append(f"{operation_id} needs a substantive local-only reason")
        bindings = exception.get("surfaces")
        if (
            not isinstance(bindings, dict)
            or not bindings
            or not set(bindings).issubset(LOCAL_SURFACES)
            or any(not _valid_binding(value) for value in bindings.values())
        ):
            errors.append(f"{operation_id} must bind only explicit local surfaces")

    state = profile.get("state")
    state_schema = state.get("schema") if isinstance(state, dict) else None
    state_schema_id = state_schema.get("id") if isinstance(state_schema, dict) else None
    if state_schema_id not in schemas or not isinstance(state_schema, dict) or state_schema.get("closed") is not True:
        errors.append("state must reference a declared closed product schema")

    registries = observed.get("registries")
    if not isinstance(registries, dict) or set(registries) != set(SURFACES):
        errors.append("observed registries must contain exactly all four product surfaces")
        registries = {}
    for surface in SURFACES:
        values = registries.get(surface)
        if not isinstance(values, list) or any(not isinstance(value, str) for value in values):
            errors.append(f"observed {surface} registry must be an array of operation IDs")
            continue
        if len(values) != len(set(values)):
            errors.append(f"observed {surface} registry contains duplicate operation IDs")
        actual = set(values)
        missing = sorted(expected[surface] - actual)
        extra = sorted(actual - expected[surface])
        if missing:
            errors.append(f"observed {surface} registry is missing: {', '.join(missing)}")
        if extra:
            errors.append(f"observed {surface} registry has undeclared operations: {', '.join(extra)}")

    return errors


def main() -> int:
    directory = Path(__file__).resolve().parent
    parser = argparse.ArgumentParser(
        description="Validate full semantic parity against generated adapter registries."
    )
    parser.add_argument(
        "profile",
        nargs="?",
        type=Path,
        default=directory / "application-profile.example.json",
    )
    parser.add_argument(
        "registry",
        nargs="?",
        type=Path,
        default=directory / "surface-registry.example.json",
    )
    arguments = parser.parse_args()
    try:
        errors = validate(
            load_json(arguments.profile),
            load_json(arguments.registry),
            arguments.profile.resolve().parent,
        )
    except (OSError, json.JSONDecodeError) as error:
        print(f"Parity validation failed: {error}", file=sys.stderr)
        return 2
    if errors:
        print("Parity validation failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print("Application profile and all four surface registries have semantic parity.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
