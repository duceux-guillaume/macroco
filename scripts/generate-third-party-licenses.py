#!/usr/bin/env python3
"""Generate THIRD_PARTY_LICENSES from Rust and Node dependencies."""

import json
import os
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
    workspace_members = set()
    for pkg_id in meta.get("workspace_members", []):
        # Extract package name from the ID string
        name = pkg_id.split("#")[0].split("/")[-1] if "#" in pkg_id else pkg_id.split()[0]
        workspace_members.add(name)

    entries = []
    for pkg in sorted(meta["packages"], key=lambda p: p["name"]):
        # Skip our own workspace crates
        if pkg["name"] in workspace_members:
            continue
        # Also skip by checking manifest path
        manifest = Path(pkg.get("manifest_path", ""))
        if str(REPO_ROOT) in str(manifest) and "/crates/" in str(manifest):
            continue

        license_text = pkg.get("license", "UNKNOWN") or "UNKNOWN"
        authors = ", ".join(pkg.get("authors", [])) or "see package repository"
        repo = pkg.get("repository", "")

        entries.append({
            "name": pkg["name"],
            "version": pkg["version"],
            "license": license_text,
            "authors": authors,
            "repository": repo,
            "ecosystem": "rust",
        })
    return entries


def node_licenses():
    """Extract licenses from node_modules package.json files."""
    frontend_dir = REPO_ROOT / "frontend"
    node_modules = frontend_dir / "node_modules"

    if not node_modules.exists():
        print("node_modules not found, skipping Node dependencies", file=sys.stderr)
        return []

    # Get full dependency tree
    result = subprocess.run(
        ["npm", "ls", "--all", "--json"],
        capture_output=True,
        text=True,
        cwd=frontend_dir,
    )
    # npm ls may exit non-zero with peer dep warnings, still produces valid JSON
    try:
        tree = json.loads(result.stdout)
    except json.JSONDecodeError:
        print("Failed to parse npm ls output", file=sys.stderr)
        return []

    def collect_deps(node, result_set):
        for name, info in node.get("dependencies", {}).items():
            ver = info.get("version", "?")
            result_set.add((name, ver))
            collect_deps(info, result_set)

    all_deps = set()
    collect_deps(tree, all_deps)

    entries = []
    for name, ver in sorted(all_deps):
        pkg_json_path = node_modules / name / "package.json"
        if not pkg_json_path.exists():
            # Try scoped package
            continue

        with open(pkg_json_path) as f:
            pkg = json.load(f)

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
            "ecosystem": "node",
        })
    return entries


def format_output(rust_entries, node_entries):
    """Format all entries into the THIRD_PARTY_LICENSES text file."""
    lines = [
        "THIRD-PARTY SOFTWARE NOTICES AND INFORMATION",
        "=" * 50,
        "",
        "Macroco incorporates third-party software components.",
        "The following is a list of these components and their licenses.",
        "",
    ]

    if rust_entries:
        lines.append("=" * 50)
        lines.append("RUST DEPENDENCIES")
        lines.append("=" * 50)
        lines.append("")
        for e in rust_entries:
            lines.append(f"  {e['name']} {e['version']}")
            lines.append(f"  License: {e['license']}")
            if e["repository"]:
                lines.append(f"  Repository: {e['repository']}")
            lines.append(f"  Authors: {e['authors']}")
            lines.append("")

    if node_entries:
        lines.append("=" * 50)
        lines.append("NODE DEPENDENCIES")
        lines.append("=" * 50)
        lines.append("")
        for e in node_entries:
            lines.append(f"  {e['name']} {e['version']}")
            lines.append(f"  License: {e['license']}")
            if e["repository"]:
                lines.append(f"  Repository: {e['repository']}")
            lines.append(f"  Authors: {e['authors']}")
            lines.append("")

    # Summary
    all_entries = rust_entries + node_entries
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
    rust = rust_licenses()
    node = node_licenses()

    if not rust and not node:
        print("No dependencies found", file=sys.stderr)
        sys.exit(1)

    output = format_output(rust, node)
    OUTPUT.write_text(output)
    print(f"Written {OUTPUT} ({len(rust)} Rust + {len(node)} Node packages)")


if __name__ == "__main__":
    main()
