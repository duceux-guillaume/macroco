#!/usr/bin/env bash
set -euo pipefail

# Agent workflow test: proves that the `diagnose` command provides enough
# structured information for a Claude Code agent to debug simulation output
# without visual chart inspection.

CLI="cargo run --release --bin world3-cli --"
PASS=0
FAIL=0

pass() { echo "  PASS  $1"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL  $1"; FAIL=$((FAIL + 1)); }

echo "=== Agent Workflow Test: diagnose command ==="
echo ""

# Build first
echo "Building CLI..."
cargo build --release --bin world3-cli 2>/dev/null

# --- Test 1: Text output contains expected sections ---
echo ""
echo "Test 1: Text output contains expected sections"
output=$($CLI diagnose --preset bau 2>/dev/null)

echo "$output" | grep -q "Simulation Diagnostics" && pass "header present" || fail "header missing"
echo "$output" | grep -q "Population" && pass "Population section" || fail "Population section missing"
echo "$output" | grep -q "Peak:" && pass "Peak line" || fail "Peak line missing"
echo "$output" | grep -q "Phases:" && pass "Phases line" || fail "Phases line missing"
echo "$output" | grep -q "Anomalies" && pass "Anomalies section" || fail "Anomalies section missing"

# --- Test 2: JSON output is valid and queryable ---
echo ""
echo "Test 2: JSON output is valid and queryable"
json=$($CLI diagnose --preset bau --format json 2>/dev/null)

echo "$json" | jq . > /dev/null 2>&1 && pass "valid JSON" || fail "invalid JSON"

var_count=$(echo "$json" | jq '.variables | length')
[ "$var_count" -eq 6 ] && pass "6 variables tracked" || fail "expected 6 variables, got $var_count"

peak_year=$(echo "$json" | jq '.variables[] | select(.name == "Population") | .peak.year')
(( $(echo "$peak_year >= 2000 && $peak_year <= 2070" | bc -l) )) && \
    pass "population peak year $peak_year in [2000, 2070]" || \
    fail "population peak year $peak_year outside range"

anomaly_count=$(echo "$json" | jq '.anomalies | length')
[ "$anomaly_count" -eq 0 ] && pass "no anomalies in BAU" || fail "unexpected anomalies: $anomaly_count"

# --- Test 3: Comparison mode produces deltas ---
echo ""
echo "Test 3: Comparison mode produces deltas"
comp=$($CLI diagnose --preset bau --compare technology 2>/dev/null)

echo "$comp" | grep -q "Comparative Diagnostics" && pass "comp header" || fail "comp header missing"
echo "$comp" | grep -q "D peak:" && pass "delta peak present" || fail "delta peak missing"
echo "$comp" | grep -q "D final:" && pass "delta final present" || fail "delta final missing"

# --- Test 4: Agent debugging scenario ---
echo ""
echo "Test 4: Agent debugging scenario"

# "Why is population declining?"
decline_phase=$(echo "$json" | jq -r '.variables[] | select(.name == "Population") | .phases[] | select(.kind == "Declining") | "\(.start_year)-\(.end_year)"')
[ -n "$decline_phase" ] && pass "extracted decline phase: $decline_phase" || fail "could not extract decline phase"

# "Is NNR monotonically declining?"
nnr_monotonic=$(echo "$json" | jq '.variables[] | select(.name == "NNR fraction") | .is_monotonic')
[ "$nnr_monotonic" = "true" ] && pass "NNR confirmed monotonic" || fail "NNR not monotonic: $nnr_monotonic"

# "What's the max pollution level?"
poll_peak=$(echo "$json" | jq '.variables[] | select(.name == "Pollution index") | .peak.value')
[ -n "$poll_peak" ] && pass "pollution peak: $poll_peak" || fail "could not extract pollution peak"

# --- Summary ---
echo ""
echo "=============================="
TOTAL=$((PASS + FAIL))
echo "Results: $PASS/$TOTAL passed"
if [ "$FAIL" -gt 0 ]; then
    echo "FAILED"
    exit 1
else
    echo "ALL PASSED"
fi
