// REQ: REQ-001
//! BAU Qualitative Dynamics Tests
//!
//! Validates that the BAU simulation reproduces World3 overshoot-and-collapse
//! dynamics via the world3_core::validation module.

mod common;

use world3_core::validation::validate_bau;

#[test]
fn bau_all_qualitative_checks_pass() {
    let sim = common::bau_sim();
    let results = validate_bau(sim);

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
        "BAU qualitative validation failed:\n{}",
        failures.join("\n")
    );
}
