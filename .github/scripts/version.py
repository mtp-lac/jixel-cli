"""Keep the CLI package version equal to its exact jixel dependency version."""

import argparse
from pathlib import Path
import re
import tomllib


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=Path("Cargo.toml"))
    parser.add_argument("--sync", metavar="VERSION", help="set both versions")
    parser.add_argument("--expected", metavar="VERSION", help="verify the release version")
    args = parser.parse_args()

    content = args.manifest.read_text(encoding="utf-8")
    manifest = tomllib.loads(content)
    if args.sync:
        if not re.fullmatch(r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)", args.sync):
            parser.error("--sync must be a stable jixel version (X.Y.Z)")

        package = re.compile(r'(?m)^(version\s*=\s*")[^"]+("[^\n]*)$')
        dependency = re.compile(r'(?m)^(jixel\s*=\s*\{[^\n}]*version\s*=\s*")[^"]+("[^\n]*)$')
        content, package_count = package.subn(lambda m: f"{m[1]}{args.sync}{m[2]}", content)
        content, dependency_count = dependency.subn(lambda m: f"{m[1]}={args.sync}{m[2]}", content)
        if package_count != 1 or dependency_count != 1:
            parser.error("expected one package version and one jixel dependency in Cargo.toml")
        manifest = tomllib.loads(content)

    version = manifest["package"]["version"]
    requirement = manifest["dependencies"]["jixel"]["version"]
    if requirement != f"={version}":
        parser.error(f"CLI version {version} requires an exact jixel ={version} pin, found {requirement}")
    if args.expected and version != args.expected:
        parser.error(f"CLI/jixel version {version} does not match release version {args.expected}")

    if args.sync:
        args.manifest.write_text(content, encoding="utf-8")
    print(version)


if __name__ == "__main__":
    main()
