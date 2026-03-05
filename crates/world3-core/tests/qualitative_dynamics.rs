// REQ: REQ-001
//! Collapse Qualitative Dynamics Tests
//!
//! Validates that the Collapse simulation reproduces World3 overshoot-and-collapse
//! dynamics via the world3_core::validation module.

mod common;

use world3_core::validation::validate_collapse;

#[test]
fn collapse_all_qualitative_checks_pass() {
    let sim = common::collapse_sim();
    let results = validate_collapse(sim);

    let mut failures = Vec::new();
    for r in &results {
        let status = if r.passed { "PASS" } else { "FAIL" };
        eprintln!("  {}  {}: {}", status, r.label, r.detail);
        if !r.passed {
            failures.push(format!("{}: {}", r.label, r.detail));
        }
    }

    assert!(
        failures.is_empty(),
        "Collapse qualitative validation failed:\n{}",
        failures.join("\n")
    );
}
