"""
Validates objectiveai-rs-pyo3 dist/ and installs the wheel.

Delegates the fingerprint check to objectiveai-rs-pyo3/validate.sh.
If dist/ is missing or stale, exits with an error — run build.sh first.
"""

import glob
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
PYO3_DIR = REPO_ROOT / "objectiveai-rs-pyo3"
DIST_DIR = PYO3_DIR / "dist"
VALIDATE_SCRIPT = PYO3_DIR / "validate.sh"


def validate() -> None:
    result = subprocess.run(
        ["bash", str(VALIDATE_SCRIPT)],
        shell=sys.platform == "win32",
    )
    if result.returncode != 0:
        print("objectiveai-rs-pyo3 dist/ is not valid. Run build.sh first.", file=sys.stderr)
        sys.exit(result.returncode)


def install_wheel() -> None:
    wheels = glob.glob(str(DIST_DIR / "*.whl"))
    if not wheels:
        print("No wheel found in dist/", file=sys.stderr)
        sys.exit(1)
    wheel = max(wheels, key=lambda w: Path(w).stat().st_mtime)
    print(f"Installing {Path(wheel).name}...")
    subprocess.check_call(
        [sys.executable, "-m", "pip", "install", wheel, "--force-reinstall", "--quiet"],
    )


def main() -> None:
    validate()
    install_wheel()


if __name__ == "__main__":
    main()
