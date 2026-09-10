from __future__ import annotations

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "scaffold_remote_control.py"


class ScaffoldTests(unittest.TestCase):
    def test_dry_run_does_not_write(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory)
            result = subprocess.run(
                [sys.executable, str(SCRIPT), str(target), "--dry-run"],
                check=False,
                capture_output=True,
                text=True,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("Would create", result.stdout)
            self.assertIn("remote-control-starter", result.stdout)
            self.assertFalse((target / "remote-control-starter").exists())

    def test_scaffolds_complete_assets_and_refuses_overwrite(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory)
            first = subprocess.run(
                [sys.executable, str(SCRIPT), str(target)],
                check=False,
                capture_output=True,
                text=True,
            )
            self.assertEqual(first.returncode, 0, first.stderr)
            created = target / "remote-control-starter"
            self.assertTrue((created / "rust" / "Cargo.toml").is_file())
            self.assertTrue((created / "browser" / "src" / "protocol.js").is_file())
            self.assertTrue((created / "contracts" / "application-profile.example.json").is_file())
            self.assertTrue((created / "contracts" / "surface-registry.example.json").is_file())
            self.assertTrue((created / "contracts" / "validate_application_profile.py").is_file())
            self.assertTrue((created / "cli" / "Cargo.toml").is_file())
            self.assertTrue((created / "cli" / "src" / "lib.rs").is_file())
            self.assertTrue((created / "cli" / "src" / "main.rs").is_file())
            self.assertTrue((created / "LICENSE").is_file())
            self.assertTrue((created / "NOTICE.md").is_file())
            self.assertTrue((created / "DEPENDENCY-LICENSES.md").is_file())
            self.assertTrue((created / "cli" / "LICENSE").is_file())
            self.assertTrue((created / "cli" / "NOTICE.md").is_file())
            self.assertIn("action/query/subscription/transfer registry", first.stdout)
            self.assertIn("browser companion", first.stdout)

            second = subprocess.run(
                [sys.executable, str(SCRIPT), str(target)],
                check=False,
                capture_output=True,
                text=True,
            )
            self.assertEqual(second.returncode, 2)
            self.assertIn("refusing to overwrite", second.stderr)

    def test_rejects_traversal_and_nested_destination_names(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            invalid_names = (
                "nested/path",
                r"nested\path",
                "../outside",
                r"..\outside",
                "C:drive-relative",
                str(Path(directory).resolve() / "absolute"),
            )
            for name in invalid_names:
                with self.subTest(name=name):
                    result = subprocess.run(
                        [
                            sys.executable,
                            str(SCRIPT),
                            directory,
                            "--destination-name",
                            name,
                        ],
                        check=False,
                        capture_output=True,
                        text=True,
                    )
                    self.assertEqual(result.returncode, 2)
                    self.assertIn("plain directory name", result.stderr)
            self.assertEqual(list(Path(directory).iterdir()), [])


if __name__ == "__main__":
    unittest.main()
