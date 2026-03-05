#!/usr/bin/env python3
"""Generate frontend/src/lib/content/variable-descriptions.ts from docs/model/ sources.

Usage:
    python3 scripts/generate-variable-descriptions.py          # write the file
    python3 scripts/generate-variable-descriptions.py --check  # exit 1 if stale
    python3 scripts/generate-variable-descriptions.py --dry-run # print to stdout
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from dataclasses import dataclass, field

ROOT = Path(__file__).resolve().parent.parent
DOCS = ROOT / "docs" / "model"
OUTPUT = ROOT / "frontend" / "src" / "lib" / "content" / "variable-descriptions.ts"

SECTOR_MAP = {
    "population": "Population",
    "capital": "Capital",
    "agriculture": "Agriculture",
    "resources": "Resources",
    "pollution": "Pollution",
}

# Parameters to skip (simulation control, not model parameters)
SKIP_PARAMS = {"start-year", "end-year", "time-step"}


# ---------------------------------------------------------------------------
# Data classes
# ---------------------------------------------------------------------------
@dataclass
class VariableInfo:
    key: str  # e.g. "population.population"
    name: str
    unit: str
    sector: str
    is_stock: bool
    beginner: str
    expert: str
    feedback_loops: list[str]
    related_variables: list[str]
    doc_path: str


@dataclass
class ParameterInfo:
    key: str  # e.g. "technology_growth_rate"
    name: str
    unit: str
    sector: str
    beginner: str
    expert: str
    feedback_loops: list[str]
    related_variables: list[str]
    impact_increase: str
    impact_decrease: str
    sparkline_variable: str
    doc_path: str


@dataclass
class FeedbackLoopInfo:
    key: str
    name: str
    loop_type: str  # "reinforcing" or "stabilizing"
    description: str
    chain: list[str]
    doc_path: str


# ---------------------------------------------------------------------------
# Parsing helpers
# ---------------------------------------------------------------------------
def extract_field(text: str, label: str) -> str:
    """Extract a **Label:** value from markdown text."""
    pattern = rf'\*\*{re.escape(label)}:\*\*\s*(.+?)(?:\n\n|\n\*\*|\n###|\n##|\Z)'
    m = re.search(pattern, text, re.DOTALL)
    if not m:
        return ""
    return m.group(1).strip()


def split_list(text: str) -> list[str]:
    """Split a comma-separated field into trimmed items."""
    if not text:
        return []
    return [item.strip() for item in text.split(",") if item.strip()]


def normalize_text(text: str) -> str:
    """Normalize markdown dashes and special chars to match TS output."""
    # Replace em-dashes (--) with real em-dashes
    text = text.replace(" -- ", " \u2014 ")
    text = text.replace("--", "\u2014")
    # Replace x with multiplication sign in math contexts
    # But be careful - only replace standalone x used as multiplication
    text = re.sub(r'(?<=[0-9)]) x (?=[A-Z0-9(])', ' \u00d7 ', text)
    text = re.sub(r'(?<=\)) x (?=\()', ' \u00d7 ', text)
    return text


def escape_ts_string(s: str) -> str:
    """Escape a string for use in a TypeScript single-quoted string literal."""
    return s.replace("\\", "\\\\").replace("'", "\\'")


# ---------------------------------------------------------------------------
# Parse feedback-loops.md
# ---------------------------------------------------------------------------
def parse_feedback_loops() -> list[FeedbackLoopInfo]:
    path = DOCS / "feedback-loops.md"
    text = path.read_text()
    doc_path = f"docs/model/feedback-loops.md"

    # Split on ## N. headings
    sections = re.split(r'^## \d+\.\s+', text, flags=re.MULTILINE)[1:]
    results = []
    for section in sections:
        # Extract name and type from first line: "Name (type)"
        first_line = section.split("\n")[0].strip()

        # Extract ID
        id_match = re.search(r'\*\*ID:\*\*\s*`([^`]+)`', section)
        if not id_match:
            continue
        loop_id = id_match.group(1)

        # Extract type
        type_match = re.search(r'\*\*Type:\*\*\s*(\w+)', section)
        loop_type = type_match.group(1) if type_match else ""

        # Extract chain
        chain_match = re.search(r'\*\*Chain:\*\*\s*(.+)', section)
        chain = []
        if chain_match:
            chain_text = chain_match.group(1).strip()
            chain = [item.strip().strip('`') for item in chain_text.split(" > ")]

        # Extract name: heading text up to " (type)"
        name_match = re.match(r'(.+?)\s*\((?:reinforcing|balancing|stabilizing)\)', first_line)
        if name_match:
            name = name_match.group(1).strip()
        else:
            name = first_line.strip()
        # Clean up markdown artifacts: --> to unicode arrow
        name = name.replace("-->", "\u2192").replace("--", "\u2014")

        # Extract description: first paragraph after **Connects:**
        connects_match = re.search(r'\*\*Connects:\*\*[^\n]*\n\n(.+?)(?:\n```|\n\n)', section, re.DOTALL)
        description = ""
        if connects_match:
            description = connects_match.group(1).strip()
        # Clean up markdown link artifacts
        description = re.sub(r'\[([^\]]+)\]\([^)]+\)', r'\1', description)

        results.append(FeedbackLoopInfo(
            key=loop_id,
            name=name,
            loop_type=loop_type,
            description=description,
            chain=chain,
            doc_path=doc_path,
        ))

    return results


# ---------------------------------------------------------------------------
# Parse sectors/*.md
# ---------------------------------------------------------------------------
def parse_sectors() -> list[VariableInfo]:
    results = []
    for sector_file in sorted((DOCS / "sectors").glob("*.md")):
        stem = sector_file.stem
        sector_name = SECTOR_MAP.get(stem, stem.capitalize())
        doc_path = f"docs/model/sectors/{sector_file.name}"

        text = sector_file.read_text()

        # Find ## Info Panel section
        info_match = re.search(r'^## Info Panel\s*\n', text, re.MULTILINE)
        if not info_match:
            continue
        info_text = text[info_match.end():]

        # Cut off at next ## heading (not ###)
        next_h2 = re.search(r'^## [^#]', info_text, re.MULTILINE)
        if next_h2:
            info_text = info_text[:next_h2.start()]

        # Split by ### headings
        var_sections = re.split(r'^### ', info_text, flags=re.MULTILINE)[1:]

        for var_section in var_sections:
            lines = var_section.split("\n", 1)
            var_key = lines[0].strip()
            body = lines[1] if len(lines) > 1 else ""

            name = extract_field(body, "Name")
            unit = extract_field(body, "Unit")
            stock_str = extract_field(body, "Stock")
            is_stock = stock_str.lower() == "true"
            beginner = extract_field(body, "Beginner")
            expert = extract_field(body, "Expert")
            feedback_loops = split_list(extract_field(body, "Feedback loops"))
            related_vars = split_list(extract_field(body, "Related variables"))

            results.append(VariableInfo(
                key=var_key,
                name=name,
                unit=unit,
                sector=sector_name,
                is_stock=is_stock,
                beginner=beginner,
                expert=expert,
                feedback_loops=feedback_loops,
                related_variables=related_vars,
                doc_path=doc_path,
            ))

    return results


# ---------------------------------------------------------------------------
# Parse parameters/*.md
# ---------------------------------------------------------------------------
def parse_parameters() -> list[ParameterInfo]:
    results = []
    for param_file in sorted((DOCS / "parameters").glob("*.md")):
        stem = param_file.stem
        if stem in SKIP_PARAMS:
            continue

        doc_path = f"docs/model/parameters/{param_file.name}"
        text = param_file.read_text()

        # Key from filename: kebab-to-snake
        key = stem.replace("-", "_")

        # Name from H1 heading
        h1_match = re.match(r'^# (.+)', text)
        name = h1_match.group(1).strip() if h1_match else stem

        # Sector from **Sector:** line (strip markdown links)
        sector_match = re.search(r'\*\*Sector:\*\*\s*(.+)', text)
        sector = ""
        if sector_match:
            sector_raw = sector_match.group(1).strip()
            # Strip markdown link: [Capital](...) -> Capital
            link_match = re.match(r'\[([^\]]+)\]\([^)]+\)', sector_raw)
            sector = link_match.group(1) if link_match else sector_raw

        # Find ## Info Panel section
        info_match = re.search(r'^## Info Panel\s*\n', text, re.MULTILINE)
        if not info_match:
            continue
        info_text = text[info_match.end():]

        # Cut off at next ## heading
        next_h2 = re.search(r'^## [^#]', info_text, re.MULTILINE)
        if next_h2:
            info_text = info_text[:next_h2.start()]

        unit = extract_field(info_text, "Unit")
        beginner = extract_field(info_text, "Beginner")
        expert = extract_field(info_text, "Expert")
        feedback_loops = split_list(extract_field(info_text, "Feedback loops"))
        related_vars = split_list(extract_field(info_text, "Related variables"))
        impact_increase = extract_field(info_text, "Impact increase")
        impact_decrease = extract_field(info_text, "Impact decrease")
        sparkline = extract_field(info_text, "Sparkline variable")

        results.append(ParameterInfo(
            key=key,
            name=name,
            unit=unit,
            sector=sector,
            beginner=beginner,
            expert=expert,
            feedback_loops=feedback_loops,
            related_variables=related_vars,
            impact_increase=impact_increase,
            impact_decrease=impact_decrease,
            sparkline_variable=sparkline,
            doc_path=doc_path,
        ))

    return results


# ---------------------------------------------------------------------------
# TypeScript generation
# ---------------------------------------------------------------------------
def ts_string(s: str) -> str:
    """Format a string as a TS single-quoted string, possibly multi-line."""
    escaped = escape_ts_string(s)
    if len(escaped) > 80:
        return f"'{escaped}'"
    return f"'{escaped}'"


def ts_string_array(items: list[str], indent: str = "\t\t") -> str:
    """Format a list of strings as a TS array."""
    if not items:
        return "[]"
    inner = ", ".join(f"'{escape_ts_string(item)}'" for item in items)
    if len(inner) + len(indent) + 4 < 100:
        return f"[{inner}]"
    lines = [f"{indent}\t'{escape_ts_string(item)}'" for item in items]
    return "[\n" + ",\n".join(lines) + f"\n{indent}]"


def generate_ts(
    variables: list[VariableInfo],
    parameters: list[ParameterInfo],
    loops: list[FeedbackLoopInfo],
) -> str:
    lines: list[str] = []

    # Header
    lines.append("/**")
    lines.append(" * AUTO-GENERATED — DO NOT EDIT")
    lines.append(" *")
    lines.append(" * Source: docs/model/ (sectors, parameters, feedback-loops)")
    lines.append(" * Generator: scripts/generate-variable-descriptions.py")
    lines.append(" */")
    lines.append("")

    # Interfaces
    lines.append("/** Structured content for chart variables, parameters, and feedback loops. */")
    lines.append("")
    lines.append("export interface VariableInfo {")
    lines.append("\tname: string;")
    lines.append("\tunit: string;")
    lines.append("\tsector: string;")
    lines.append("\tisStock: boolean;")
    lines.append("\tbeginner: string;")
    lines.append("\texpert: string;")
    lines.append("\tfeedbackLoops: string[];")
    lines.append("\trelatedVariables: string[];")
    lines.append("\tdocPath?: string;")
    lines.append("}")
    lines.append("")
    lines.append("export interface ParameterImpact {")
    lines.append("\tincrease: string;")
    lines.append("\tdecrease: string;")
    lines.append("\tsparklineVariable: string;")
    lines.append("}")
    lines.append("")
    lines.append("export interface ParameterInfo {")
    lines.append("\tname: string;")
    lines.append("\tunit: string;")
    lines.append("\tsector: string;")
    lines.append("\tbeginner: string;")
    lines.append("\texpert: string;")
    lines.append("\tfeedbackLoops: string[];")
    lines.append("\trelatedVariables: string[];")
    lines.append("\timpact: ParameterImpact;")
    lines.append("\tdocPath?: string;")
    lines.append("}")
    lines.append("")
    lines.append("export interface FeedbackLoopInfo {")
    lines.append("\tid: string;")
    lines.append("\tname: string;")
    lines.append("\ttype: 'reinforcing' | 'stabilizing';")
    lines.append("\tdescription: string;")
    lines.append("\tchain: string[];")
    lines.append("\tdocPath?: string;")
    lines.append("}")
    lines.append("")

    # Variable descriptions
    lines.append("/** Variable descriptions keyed by ChartConfig.fieldPath */")
    lines.append("export const variableDescriptions: Record<string, VariableInfo> = {")

    for i, v in enumerate(variables):
        lines.append(f"\t'{escape_ts_string(v.key)}': {{")
        lines.append(f"\t\tname: '{escape_ts_string(v.name)}',")
        lines.append(f"\t\tunit: '{escape_ts_string(v.unit)}',")
        lines.append(f"\t\tsector: '{escape_ts_string(v.sector)}',")
        lines.append(f"\t\tisStock: {'true' if v.is_stock else 'false'},")
        lines.append(f"\t\tbeginner:")
        lines.append(f"\t\t\t'{escape_ts_string(v.beginner)}',")
        lines.append(f"\t\texpert:")
        lines.append(f"\t\t\t'{escape_ts_string(v.expert)}',")
        lines.append(f"\t\tfeedbackLoops: {ts_string_array(v.feedback_loops)},")
        lines.append(f"\t\trelatedVariables: {ts_string_array(v.related_variables)},")
        lines.append(f"\t\tdocPath: '{escape_ts_string(v.doc_path)}'")
        trail = "," if i < len(variables) - 1 else ""
        lines.append(f"\t}}{trail}")
    lines.append("};")
    lines.append("")

    # Parameter descriptions
    lines.append("/** Parameter descriptions keyed by ScenarioParams field name */")
    lines.append("export const parameterDescriptions: Record<string, ParameterInfo> = {")

    for i, p in enumerate(parameters):
        lines.append(f"\t{p.key}: {{")
        lines.append(f"\t\tname: '{escape_ts_string(p.name)}',")
        lines.append(f"\t\tunit: '{escape_ts_string(p.unit)}',")
        lines.append(f"\t\tsector: '{escape_ts_string(p.sector)}',")
        lines.append(f"\t\tbeginner:")
        lines.append(f"\t\t\t'{escape_ts_string(p.beginner)}',")
        lines.append(f"\t\texpert:")
        lines.append(f"\t\t\t'{escape_ts_string(p.expert)}',")
        lines.append(f"\t\tfeedbackLoops: {ts_string_array(p.feedback_loops)},")
        lines.append(f"\t\trelatedVariables: {ts_string_array(p.related_variables)},")
        lines.append(f"\t\timpact: {{")
        lines.append(f"\t\t\tincrease: '{escape_ts_string(p.impact_increase)}',")
        lines.append(f"\t\t\tdecrease: '{escape_ts_string(p.impact_decrease)}',")
        lines.append(f"\t\t\tsparklineVariable: '{escape_ts_string(p.sparkline_variable)}'")
        lines.append(f"\t\t}},")
        lines.append(f"\t\tdocPath: '{escape_ts_string(p.doc_path)}'")
        trail = "," if i < len(parameters) - 1 else ""
        lines.append(f"\t}}{trail}")
    lines.append("};")
    lines.append("")

    # Feedback loops
    lines.append("/** Major feedback loops connecting the sectors */")
    lines.append("export const feedbackLoops: Record<string, FeedbackLoopInfo> = {")

    for i, loop in enumerate(loops):
        lines.append(f"\t'{escape_ts_string(loop.key)}': {{")
        lines.append(f"\t\tid: '{escape_ts_string(loop.key)}',")
        lines.append(f"\t\tname: '{escape_ts_string(loop.name)}',")
        lines.append(f"\t\ttype: '{loop.loop_type}',")
        lines.append(f"\t\tdescription:")
        lines.append(f"\t\t\t'{escape_ts_string(loop.description)}',")
        lines.append(f"\t\tchain: {ts_string_array(loop.chain)},")
        lines.append(f"\t\tdocPath: '{escape_ts_string(loop.doc_path)}'")
        trail = "," if i < len(loops) - 1 else ""
        lines.append(f"\t}}{trail}")
    lines.append("};")
    lines.append("")

    # getRelatedParameters utility
    lines.append("/** Reverse index: for a given variable path, find all parameters that list it in relatedVariables. */")
    lines.append("const relatedParamsCache = new Map<string, Array<{ path: string; name: string }>>();")
    lines.append("")
    lines.append("export function getRelatedParameters(variablePath: string): Array<{ path: string; name: string }> {")
    lines.append("\tconst cached = relatedParamsCache.get(variablePath);")
    lines.append("\tif (cached) return cached;")
    lines.append("")
    lines.append("\tconst result: Array<{ path: string; name: string }> = [];")
    lines.append("\tfor (const [key, param] of Object.entries(parameterDescriptions)) {")
    lines.append("\t\tif (param.relatedVariables.includes(variablePath)) {")
    lines.append("\t\t\tresult.push({ path: key, name: param.name });")
    lines.append("\t\t}")
    lines.append("\t}")
    lines.append("\trelatedParamsCache.set(variablePath, result);")
    lines.append("\treturn result;")
    lines.append("}")
    lines.append("")

    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
def main() -> int:
    parser = argparse.ArgumentParser(description="Generate variable-descriptions.ts from docs/model/")
    parser.add_argument("--check", action="store_true", help="Exit 1 if generated output differs from existing file")
    parser.add_argument("--dry-run", action="store_true", help="Print to stdout instead of writing")
    args = parser.parse_args()

    variables = parse_sectors()
    parameters = parse_parameters()
    loops = parse_feedback_loops()

    output = generate_ts(variables, parameters, loops)

    if args.dry_run:
        print(output)
        return 0

    if args.check:
        if not OUTPUT.exists():
            print(f"ERROR: {OUTPUT} does not exist", file=sys.stderr)
            return 1
        existing = OUTPUT.read_text()
        if existing == output:
            print("OK: variable-descriptions.ts is up to date")
            return 0
        else:
            print(f"ERROR: {OUTPUT} is stale. Run: python3 scripts/generate-variable-descriptions.py", file=sys.stderr)
            return 1

    OUTPUT.write_text(output)
    print(f"Wrote {OUTPUT}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
