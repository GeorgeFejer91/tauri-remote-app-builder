from __future__ import annotations

import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from scripts import check_repository


ROOT = Path(__file__).resolve().parents[1]


class RepositoryTests(unittest.TestCase):
    def test_repository_validator_passes(self) -> None:
        result = subprocess.run(
            [sys.executable, str(ROOT / "scripts" / "check_repository.py")],
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("validation passed", result.stdout)

    def test_full_tauri_reference_library_is_required(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            copied = Path(directory) / "skill"
            shutil.copytree(
                ROOT,
                copied,
                ignore=shutil.ignore_patterns(".git", "__pycache__", "node_modules", "target"),
            )
            missing = copied / "references" / "tauri" / "rust-quality.md"
            missing.unlink()

            errors = check_repository.validate_repository(copied)

            self.assertIn(
                "missing required file: references/tauri/rust-quality.md",
                errors,
            )

    def test_pretext_policy_and_license_are_required(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            copied = Path(directory) / "skill"
            shutil.copytree(
                ROOT,
                copied,
                ignore=shutil.ignore_patterns(".git", "__pycache__", "node_modules", "target"),
            )
            (copied / "references" / "pretext-text-layout.md").unlink()
            (copied / "licenses" / "PRETEXT-LICENSE").unlink()

            errors = check_repository.validate_repository(copied)

            self.assertIn(
                "missing required file: references/pretext-text-layout.md",
                errors,
            )
            self.assertIn("missing required file: licenses/PRETEXT-LICENSE", errors)

    def test_invalid_json_and_every_local_markdown_link_form_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            copied = Path(directory) / "skill"
            shutil.copytree(
                ROOT,
                copied,
                ignore=shutil.ignore_patterns(".git", "__pycache__", "node_modules", "target"),
            )
            checks = copied / "validation-fixtures"
            checks.mkdir()
            (checks / "invalid.json").write_text('{"broken":', encoding="utf-8")
            (checks / "links.md").write_text(
                "![missing image](missing.png)\n"
                "[missing document][missing-doc]\n"
                "[missing-doc]: missing-document.md\n"
                "[escape]: ../../outside.md\n",
                encoding="utf-8",
            )

            errors = check_repository.validate_repository(copied)

            self.assertTrue(
                any("invalid JSON validation-fixtures/invalid.json" in error for error in errors)
            )
            self.assertIn(
                "broken local link in validation-fixtures/links.md: "
                "validation-fixtures/missing.png",
                errors,
            )
            self.assertIn(
                "broken local link in validation-fixtures/links.md: "
                "validation-fixtures/missing-document.md",
                errors,
            )
            self.assertTrue(
                any(
                    error.startswith(
                        "local link escapes repository in validation-fixtures/links.md:"
                    )
                    for error in errors
                )
            )

    def test_combined_skill_identity_and_invocation_are_enforced(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            copied = Path(directory) / "skill"
            shutil.copytree(
                ROOT,
                copied,
                ignore=shutil.ignore_patterns(".git", "__pycache__", "node_modules", "target"),
            )
            skill_path = copied / "SKILL.md"
            skill_path.write_text(
                skill_path.read_text(encoding="utf-8").replace(
                    "name: tauri-remote-app-builder",
                    "name: old-skill-name",
                    1,
                ),
                encoding="utf-8",
            )
            yaml_path = copied / "agents" / "openai.yaml"
            yaml_path.write_text(
                yaml_path.read_text(encoding="utf-8").replace(
                    "$tauri-remote-app-builder",
                    "$old-skill-name",
                    1,
                ),
                encoding="utf-8",
            )

            errors = check_repository.validate_repository(copied)

            self.assertIn(
                "unexpected skill name: expected tauri-remote-app-builder",
                errors,
            )
            self.assertIn(
                "openai.yaml default prompt must name $tauri-remote-app-builder",
                errors,
            )

    def test_profile_requires_full_browser_cli_query_stream_and_transfer_parity(self) -> None:
        profile = json.loads(
            (
                ROOT
                / "assets"
                / "starter"
                / "contracts"
                / "application-profile.example.json"
            ).read_text(encoding="utf-8")
        )
        profile["remoteControl"]["policy"] = "local-only"
        del profile["operations"][0]["surfaces"]["browserCompanion"]

        errors = check_repository.validate_profile(profile)

        self.assertIn(
            "example profile must declare full-semantic-parity remote control",
            errors,
        )
        self.assertTrue(
            any(
                "must bind tauri, localCli, remoteCli, and browserCompanion" in error
                for error in errors
            )
        )

    def test_profile_schema_requires_unified_registry_and_surface_contracts(self) -> None:
        schema = json.loads(
            (
                ROOT
                / "assets"
                / "starter"
                / "contracts"
                / "application-profile.schema.json"
            ).read_text(encoding="utf-8")
        )
        schema["$defs"]["surfaceBindings"]["required"].remove("browserCompanion")
        schema["$defs"]["operation"]["required"].remove("idempotency")

        errors = check_repository.validate_profile_schema(schema)

        self.assertIn(
            "application profile schema must require complete operation metadata",
            errors,
        )
        self.assertIn(
            "shared operations must require all four concrete adapter bindings",
            errors,
        )

    def test_scaffolded_parity_validator_rejects_registry_drift(self) -> None:
        contracts = ROOT / "assets" / "starter" / "contracts"
        profile = json.loads(
            (contracts / "application-profile.example.json").read_text(encoding="utf-8")
        )
        registry = json.loads(
            (contracts / "surface-registry.example.json").read_text(encoding="utf-8")
        )
        registry["registries"]["browserCompanion"].remove("content_upload")

        namespace: dict[str, object] = {"__name__": "profile_validator_test"}
        exec(
            (contracts / "validate_application_profile.py").read_text(encoding="utf-8"),
            namespace,
        )
        errors = namespace["validate"](profile, registry)

        self.assertIn(
            "observed browserCompanion registry is missing: content_upload",
            errors,
        )


if __name__ == "__main__":
    unittest.main()
