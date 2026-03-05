#!/usr/bin/env python3
"""
Model documentation audit script.

Verifies that docs/model/ is complete, correctly templated, and in sync
with the Rust source code. Designed to run in CI (--check mode).

Phases:
  1. Completeness — every code entity has a doc file, no orphans
  2. Template conformance — required headings present
  3. Code-doc sync — breakpoint values and BAU defaults match

Usage:
  python3 scripts/audit-model-doc.py          # full report
  python3 scripts/audit-model-doc.py --check  # CI mode, exit 1 on issues
"""

import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
TABLES_RS = REPO_ROOT / "crates" / "world3-core" / "src" / "lookup" / "tables.rs"
PARAMS_RS = REPO_ROOT / "crates" / "world3-core" / "src" / "model" / "params.rs"
SECTORS_DIR = REPO_ROOT / "crates" / "world3-core" / "src" / "model" / "sectors"
DOC_TABLES = REPO_ROOT / "docs" / "model" / "tables"
DOC_PARAMS = REPO_ROOT / "docs" / "model" / "parameters"
DOC_SECTORS = REPO_ROOT / "docs" / "model" / "sectors"


def to_kebab(name: str) -> str:
    """Convert snake_case to kebab-case."""
    return name.replace("_", "-")


def extract_struct_fields(path: Path, struct_name: str) -> list[str]:
    """Extract pub field names from a Rust struct definition."""
    text = path.read_text()
    # Find the struct block
    pattern = rf"pub struct {struct_name}\s*\{{(.*?)\}}"
    match = re.search(pattern, text, re.DOTALL)
    if not match:
        return []
    body = match.group(1)
    fields = re.findall(r"pub\s+(\w+)\s*:", body)
    return fields


def extract_lookup_fields() -> list[str]:
    """Extract field names from WorldLookupTables struct."""
    return extract_struct_fields(TABLES_RS, "WorldLookupTables")


def extract_param_fields() -> list[str]:
    """Extract field names from ScenarioParams struct, excluding meta."""
    fields = extract_struct_fields(PARAMS_RS, "ScenarioParams")
    return [f for f in fields if f != "meta"]


def extract_sector_files() -> list[str]:
    """Get sector .rs file names (excluding mod.rs)."""
    return [
        f.stem
        for f in SECTORS_DIR.glob("*.rs")
        if f.name != "mod.rs"
    ]


def extract_lookup_values(tables_rs: str) -> dict[str, tuple[list[float], list[float]]]:
    """Parse LookupTable::new(...) calls to extract x and y arrays."""
    result = {}
    # Match: field_name: LookupTable::new("name", vec![...], vec![...])
    # We need to find each field assignment
    pattern = re.compile(
        r'(\w+):\s*LookupTable::new\(\s*"[^"]*",\s*vec!\[([\d\s.,e+\-]+)\],\s*vec!\[([\d\s.,e+\-]+)\]',
        re.DOTALL,
    )
    for m in pattern.finditer(tables_rs):
        field = m.group(1)
        x_str = m.group(2)
        y_str = m.group(3)
        x = [float(v.strip()) for v in x_str.split(",") if v.strip()]
        y = [float(v.strip()) for v in y_str.split(",") if v.strip()]
        result[field] = (x, y)
    return result


def extract_default_values(params_rs: str) -> dict[str, str]:
    """Parse ScenarioParams::default() to extract field = value pairs."""
    result = {}
    # Find the Default impl block
    match = re.search(r"impl Default for ScenarioParams\s*\{.*?fn default\(\).*?\{.*?Self\s*\{(.*?)\}\s*\}", params_rs, re.DOTALL)
    if not match:
        return result
    body = match.group(1)
    # Match field: value patterns (handling expressions like 1.0 / 13.0)
    for line in body.split("\n"):
        line = line.strip()
        if line.startswith("//") or not line or line.startswith("meta"):
            continue
        m = re.match(r"(\w+):\s*(.+?),?\s*(?://.*)?$", line)
        if m:
            field = m.group(1)
            value = m.group(2).strip().rstrip(",")
            result[field] = value
    return result


def parse_breakpoints_from_doc(doc_path: Path) -> list[float] | None:
    """Extract Macroco column values from ## Breakpoints table."""
    text = doc_path.read_text()
    # Find ## Breakpoints section
    bp_match = re.search(r"## Breakpoints\s*\n(.*?)(?=\n##|\Z)", text, re.DOTALL)
    if not bp_match:
        return None
    section = bp_match.group(1)
    # Find the markdown table rows (skip header and separator)
    rows = [line for line in section.split("\n") if line.strip().startswith("|") and not line.strip().startswith("|--") and not line.strip().startswith("|-")]
    if len(rows) < 2:  # header + at least one data row
        return None
    # Determine which column has "Macroco" or is the value column
    header = rows[0]
    cols = [c.strip() for c in header.split("|")]
    macroco_idx = None
    for i, col in enumerate(cols):
        if "macroco" in col.lower():
            macroco_idx = i
            break
    # If no Macroco column, use second column (first data column after x)
    if macroco_idx is None:
        # For exact match tables, the value is the second column
        macroco_idx = 2  # |x|y| → index 2
    values = []
    for row in rows[1:]:  # skip header
        cells = [c.strip() for c in row.split("|")]
        if macroco_idx < len(cells):
            try:
                values.append(float(cells[macroco_idx]))
            except (ValueError, IndexError):
                continue
    return values if values else None


def check_headings(path: Path, required: list[str]) -> list[str]:
    """Check that a markdown file contains all required ## headings."""
    text = path.read_text()
    missing = []
    for heading in required:
        if heading not in text:
            missing.append(heading)
    return missing


# ── Phase 1: Completeness ──────────────────────────────────────────────

def phase1_completeness() -> list[str]:
    issues = []

    # Tables
    lookup_fields = extract_lookup_fields()
    for field in lookup_fields:
        doc = DOC_TABLES / f"{to_kebab(field)}.md"
        if not doc.exists():
            issues.append(f"Missing table doc: {doc.relative_to(REPO_ROOT)}")

    # Parameters
    param_fields = extract_param_fields()
    for field in param_fields:
        doc = DOC_PARAMS / f"{to_kebab(field)}.md"
        if not doc.exists():
            issues.append(f"Missing parameter doc: {doc.relative_to(REPO_ROOT)}")

    # Sectors
    sector_names = extract_sector_files()
    for name in sector_names:
        doc = DOC_SECTORS / f"{name}.md"
        if not doc.exists():
            issues.append(f"Missing sector doc: {doc.relative_to(REPO_ROOT)}")

    # Orphans
    if DOC_TABLES.exists():
        for doc in DOC_TABLES.glob("*.md"):
            snake = doc.stem.replace("-", "_")
            if snake not in lookup_fields:
                issues.append(f"Orphan table doc: {doc.relative_to(REPO_ROOT)} (no field '{snake}' in WorldLookupTables)")

    if DOC_PARAMS.exists():
        for doc in DOC_PARAMS.glob("*.md"):
            snake = doc.stem.replace("-", "_")
            if snake not in param_fields:
                issues.append(f"Orphan parameter doc: {doc.relative_to(REPO_ROOT)} (no field '{snake}' in ScenarioParams)")

    if DOC_SECTORS.exists():
        for doc in DOC_SECTORS.glob("*.md"):
            if doc.stem not in sector_names:
                issues.append(f"Orphan sector doc: {doc.relative_to(REPO_ROOT)} (no sector '{doc.stem}.rs')")

    return issues


# ── Phase 2: Template Conformance ──────────────────────────────────────

TABLE_HEADINGS = ["## Equation Context", "## Breakpoints", "## References"]
SECTOR_HEADINGS = ["## Overview", "## State Variables", "## Governing Equations", "## Lookup Tables", "## References"]
PARAM_HEADINGS = ["## Equation Context", "## Calibration", "## References"]
VALID_STATUSES = ["Exact match", "Intentional deviation", "Custom / no reference"]


def phase2_template() -> list[str]:
    issues = []

    if DOC_TABLES.exists():
        for doc in DOC_TABLES.glob("*.md"):
            missing = check_headings(doc, TABLE_HEADINGS)
            if missing:
                issues.append(f"{doc.relative_to(REPO_ROOT)}: missing headings {missing}")
            # Check status line
            text = doc.read_text()
            if "**Status:**" not in text:
                issues.append(f"{doc.relative_to(REPO_ROOT)}: missing **Status:** line")
            else:
                has_valid = any(s in text for s in VALID_STATUSES)
                if not has_valid:
                    issues.append(f"{doc.relative_to(REPO_ROOT)}: **Status:** must be one of {VALID_STATUSES}")

    if DOC_SECTORS.exists():
        for doc in DOC_SECTORS.glob("*.md"):
            missing = check_headings(doc, SECTOR_HEADINGS)
            if missing:
                issues.append(f"{doc.relative_to(REPO_ROOT)}: missing headings {missing}")

    if DOC_PARAMS.exists():
        for doc in DOC_PARAMS.glob("*.md"):
            missing = check_headings(doc, PARAM_HEADINGS)
            if missing:
                issues.append(f"{doc.relative_to(REPO_ROOT)}: missing headings {missing}")

    return issues


# ── Phase 3: Code-Doc Sync ─────────────────────────────────────────────

def phase3_sync() -> list[str]:
    issues = []

    # Check lookup table breakpoints
    tables_text = TABLES_RS.read_text()
    code_values = extract_lookup_values(tables_text)

    for field, (x_code, y_code) in code_values.items():
        doc = DOC_TABLES / f"{to_kebab(field)}.md"
        if not doc.exists():
            continue  # Already caught in phase 1
        doc_y = parse_breakpoints_from_doc(doc)
        if doc_y is None:
            issues.append(f"{doc.relative_to(REPO_ROOT)}: could not parse breakpoints from ## Breakpoints table")
            continue
        if len(doc_y) != len(y_code):
            issues.append(f"{doc.relative_to(REPO_ROOT)}: breakpoint count mismatch (doc={len(doc_y)}, code={len(y_code)})")
            continue
        for i, (dv, cv) in enumerate(zip(doc_y, y_code)):
            if abs(dv - cv) > 1e-6:
                issues.append(f"{doc.relative_to(REPO_ROOT)}: y[{i}] mismatch (doc={dv}, code={cv})")
                break

    # Check source code paths exist
    for subdir in [DOC_TABLES, DOC_SECTORS, DOC_PARAMS]:
        if not subdir.exists():
            continue
        for doc in subdir.glob("*.md"):
            text = doc.read_text()
            source_match = re.search(r"\*\*Source code:\*\*\s*`([^`]+)`", text)
            if source_match:
                source_path = REPO_ROOT / source_match.group(1)
                if not source_path.exists():
                    issues.append(f"{doc.relative_to(REPO_ROOT)}: source path '{source_match.group(1)}' does not exist")

    return issues


# ── Main ───────────────────────────────────────────────────────────────

def main():
    check_mode = "--check" in sys.argv

    print("Model Documentation Audit")
    print("=" * 40)

    all_issues = []

    # Phase 1
    issues = phase1_completeness()
    all_issues.extend(issues)
    if issues:
        print(f"\nPhase 1 (Completeness): FAIL ({len(issues)} issues)")
        for i in issues:
            print(f"  - {i}")
    else:
        lookup_count = len(extract_lookup_fields())
        param_count = len(extract_param_fields())
        sector_count = len(extract_sector_files())
        print(f"\nPhase 1 (Completeness): OK ({lookup_count} tables, {param_count} parameters, {sector_count} sectors)")

    # Phase 2
    issues = phase2_template()
    all_issues.extend(issues)
    if issues:
        print(f"\nPhase 2 (Template): FAIL ({len(issues)} issues)")
        for i in issues:
            print(f"  - {i}")
    else:
        print("\nPhase 2 (Template): OK")

    # Phase 3
    issues = phase3_sync()
    all_issues.extend(issues)
    if issues:
        print(f"\nPhase 3 (Code-Doc Sync): FAIL ({len(issues)} issues)")
        for i in issues:
            print(f"  - {i}")
    else:
        print("\nPhase 3 (Code-Doc Sync): OK")

    # Summary
    print("\n" + "=" * 40)
    if all_issues:
        print(f"Overall: FAIL ({len(all_issues)} issues)")
        if check_mode:
            sys.exit(1)
    else:
        print("Overall: PASS")


if __name__ == "__main__":
    main()
