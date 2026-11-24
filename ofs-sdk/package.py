"""Smoke-test the native library, then bundle it with the public ABI and example."""
from pathlib import Path
import subprocess
import sys
from zipfile import ZIP_DEFLATED, ZipFile

sdk = Path(__file__).resolve().parent
library_dir = Path(sys.argv[1]).resolve()
library_name = "ofs_sdk.dll" if sys.platform == "win32" else "libofs_sdk.so"
library = library_dir / library_name
subprocess.run([sys.executable, str(sdk / "example.py"), str(library)], check=True)
with ZipFile(sys.argv[2], "w", ZIP_DEFLATED) as archive:
    for path in [library, sdk / "include/ofs_sdk.h", sdk / "example.py", sdk / "README.md", sdk.parent / "LICENSE"]:
        archive.write(path, path.name)
    if sys.platform == "win32":
        archive.write(library_dir / "ofs_sdk.dll.lib", "ofs_sdk.lib")
