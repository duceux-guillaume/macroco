#!/usr/bin/env python3
"""Generate THIRD_PARTY_LICENSES from Rust and Node dependencies."""

import argparse
import json
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
OUTPUT = REPO_ROOT / "THIRD_PARTY_LICENSES"


def rust_licenses():
    """Extract licenses from cargo metadata."""
    result = subprocess.run(
        ["cargo", "metadata", "--format-version", "1"],
        capture_output=True,
        text=True,
        cwd=REPO_ROOT,
    )
    if result.returncode != 0:
        print(f"cargo metadata failed: {result.stderr}", file=sys.stderr)
        return []

    meta = json.loads(result.stdout)
    repo_root_str = str(REPO_ROOT)

    entries = []
    for pkg in sorted(meta["packages"], key=lambda p: p["name"]):
        # Skip workspace crates by checking manifest path
        if str(pkg.get("manifest_path", "")).startswith(repo_root_str):
            continue

        license_text = pkg.get("license", "UNKNOWN") or "UNKNOWN"
        authors = ", ".join(pkg.get("authors", [])) or "see package repository"

        entries.append({
            "name": pkg["name"],
            "version": pkg["version"],
            "license": license_text,
            "authors": authors,
            "repository": pkg.get("repository", ""),
        })
    return entries


def node_licenses():
    """Extract licenses from node_modules package.json files."""
    frontend_dir = REPO_ROOT / "frontend"
    node_modules = frontend_dir / "node_modules"

    if not node_modules.exists():
        print("node_modules not found, skipping Node dependencies", file=sys.stderr)
        return []

    # npm ls may exit non-zero with peer dep warnings, still produces valid JSON
    proc = subprocess.run(
        ["npm", "ls", "--all", "--json"],
        capture_output=True,
        text=True,
        cwd=frontend_dir,
    )
    try:
        tree = json.loads(proc.stdout)
    except json.JSONDecodeError:
        print("Failed to parse npm ls output", file=sys.stderr)
        return []

    def collect_deps(node, deps):
        for name, info in node.get("dependencies", {}).items():
            deps.add((name, info.get("version", "?")))
            collect_deps(info, deps)

    all_deps = set()
    collect_deps(tree, all_deps)

    entries = []
    for name, ver in sorted(all_deps):
        pkg_json_path = node_modules / name / "package.json"
        try:
            with open(pkg_json_path) as f:
                pkg = json.load(f)
        except (OSError, json.JSONDecodeError):
            continue

        lic = pkg.get("license", "UNKNOWN")
        if isinstance(lic, dict):
            lic = lic.get("type", "UNKNOWN")

        author = pkg.get("author", "")
        if isinstance(author, dict):
            author = author.get("name", "")
        repo = pkg.get("repository", "")
        if isinstance(repo, dict):
            repo = repo.get("url", "")

        entries.append({
            "name": name,
            "version": ver,
            "license": lic or "UNKNOWN",
            "authors": author or "see package repository",
            "repository": repo or "",
        })
    return entries


def _format_section(lines, title, entries):
    """Append a titled section of license entries to lines."""
    if not entries:
        return
    lines.append("=" * 50)
    lines.append(title)
    lines.append("=" * 50)
    lines.append("")
    for e in entries:
        lines.append(f"  {e['name']} {e['version']}")
        lines.append(f"  License: {e['license']}")
        if e["repository"]:
            lines.append(f"  Repository: {e['repository']}")
        lines.append(f"  Authors: {e['authors']}")
        lines.append("")


def format_output(sections):
    """Format all entries into the THIRD_PARTY_LICENSES text file."""
    lines = [
        "THIRD-PARTY SOFTWARE NOTICES AND INFORMATION",
        "=" * 50,
        "",
        "Macroco incorporates third-party software components.",
        "The following is a list of these components and their licenses.",
        "",
    ]

    all_entries = []
    for title, entries in sections:
        _format_section(lines, title, entries)
        all_entries.extend(entries)

    # Summary
    license_counts = {}
    for e in all_entries:
        lic = e["license"]
        license_counts[lic] = license_counts.get(lic, 0) + 1

    lines.append("=" * 50)
    lines.append("LICENSE SUMMARY")
    lines.append("=" * 50)
    lines.append("")
    for lic, count in sorted(license_counts.items(), key=lambda x: -x[1]):
        lines.append(f"  {lic}: {count}")
    lines.append("")
    lines.append(f"Total third-party packages: {len(all_entries)}")
    lines.append("")

    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description="Generate THIRD_PARTY_LICENSES")
    parser.add_argument(
        "--check",
        action="store_true",
        help="Check that committed file is up to date (exit 1 if stale)",
    )
    args = parser.parse_args()

    rust = rust_licenses()
    node = node_licenses()

    if not rust and not node:
        print("No dependencies found", file=sys.stderr)
        sys.exit(1)

    output = format_output([
        ("RUST DEPENDENCIES", rust),
        ("NODE DEPENDENCIES", node),
    ])

    if args.check:
        if not OUTPUT.exists():
            print(f"FAIL: {OUTPUT} does not exist. Run without --check to generate.", file=sys.stderr)
            sys.exit(1)
        existing = OUTPUT.read_text()
        if existing != output:
            print(f"FAIL: {OUTPUT} is stale. Run `python3 scripts/generate-third-party-licenses.py` to update.", file=sys.stderr)
            sys.exit(1)
        print(f"OK: {OUTPUT} is up to date ({len(rust)} Rust + {len(node)} Node packages)")
        return

    OUTPUT.write_text(output)
    print(f"Written {OUTPUT} ({len(rust)} Rust + {len(node)} Node packages)")


if __name__ == "__main__":
    main()
