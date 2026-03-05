mod diagnose;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use world3_core::{
    model::{
        params::ScenarioParams,
        state::WorldState,
    },
    output::SimulationOutput,
    solver::traits::OdeSolver,
    Rk4Solver,
};

#[derive(Parser)]
#[command(name = "world3-cli", about = "World 3 system dynamics simulation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a simulation and output results
    Simulate {
        /// Preset scenario: bau, technology, stabilized
        #[arg(long, default_value = "bau")]
        preset: String,

        /// Output CSV file path (prints summary to stdout if omitted)
        #[arg(long)]
        output: Option<PathBuf>,

        /// Start year
        #[arg(long, default_value_t = 1900.0)]
        start: f64,

        /// End year
        #[arg(long, default_value_t = 2100.0)]
        end: f64,

        /// Time step (years)
        #[arg(long, default_value_t = 1.0)]
        dt: f64,
    },

    /// Validate BAU run against Meadows 1972 reference checkpoints
    Validate,

    /// List all available presets
    Presets,

    /// Run simulation diagnostics -- structured text/JSON analysis for debugging
    Diagnose {
        /// Preset scenario: bau, technology, stabilized
        #[arg(long, default_value = "bau")]
        preset: String,

        /// Compare against a second preset
        #[arg(long)]
        compare: Option<String>,

        /// Output format: text or json
        #[arg(long, default_value = "text", value_parser = ["text", "json"])]
        format: String,

        /// Start year
        #[arg(long, default_value_t = 1900.0)]
        start: f64,

        /// End year
        #[arg(long, default_value_t = 2100.0)]
        end: f64,

        /// Time step (years)
        #[arg(long, default_value_t = 1.0)]
        dt: f64,

        /// Run dt-sensitivity stability check (tests dt, dt/2, dt/4)
        #[arg(long)]
        stability_check: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Simulate { preset, output, start, end, dt } => {
            let mut params = preset_params(&preset)?;
            params.start_year = start;
            params.end_year = end;
            params.time_step = dt;

            eprintln!("Running '{}' ({} → {}, dt={}yr)…", params.meta.name, start, end, dt);

            let initial = WorldState::initial_1900();
            let tables = std::sync::Arc::new(
                world3_core::lookup::tables::WorldLookupTables::load(),
            );
            let solver = Rk4Solver::new(tables);
            let states = solver.solve(initial, &params)?;
            let sim = SimulationOutput::new(states, params);

            eprintln!(
                "Completed {} steps. Final year: {:.0}",
                sim.states.len(),
                sim.timeline.last().copied().unwrap_or(0.0)
            );

            if let Some(path) = output {
                write_csv(&sim, &path)?;
                eprintln!("Wrote {}", path.display());
            } else {
                print_summary(&sim);
            }
        }

        Commands::Validate => {
            validate()?;
        }

        Commands::Presets => {
            println!("Available presets:");
            println!("  bau          Business as Usual (original World 3 standard run)");
            println!("  technology   Comprehensive Technology scenario");
            println!("  stabilized   Stabilized World scenario");
        }

        Commands::Diagnose { preset, compare: compare_preset, format, start, end, dt, stability_check } => {
            if stability_check {
                eprintln!("Running stability check for '{}' (dt={}, {}, {})...", preset, dt, dt / 2.0, dt / 4.0);
                let report = diagnose::run_stability_check(&preset, start, end, dt)?;
                let output = match format.as_str() {
                    "json" => diagnose::format_json::format_json_stability(&report),
                    _ => diagnose::format_text::format_text_stability(&report),
                };
                println!("{}", output);
            } else {
                eprintln!("Running diagnostics for '{}'...", preset);
                let diag = diagnose::run_analysis(&preset, start, end, dt)?;

                if let Some(ref comp_name) = compare_preset {
                    eprintln!("Running comparison against '{}'...", comp_name);
                    let comp_diag = diagnose::run_analysis(comp_name, start, end, dt)?;
                    let comparative = diagnose::compare::compare(diag, comp_diag);
                    let output = match format.as_str() {
                        "json" => diagnose::format_json::format_json_comparative(&comparative),
                        _ => diagnose::format_text::format_text_comparative(&comparative),
                    };
                    println!("{}", output);
                } else {
                    let output = match format.as_str() {
                        "json" => diagnose::format_json::format_json(&diag),
                        _ => diagnose::format_text::format_text(&diag),
                    };
                    println!("{}", output);
                }
            }
        }
    }

    Ok(())
}

fn preset_params(name: &str) -> Result<ScenarioParams> {
    match name {
        "bau" => Ok(ScenarioParams::bau()),
        "technology" => Ok(ScenarioParams::comprehensive_technology()),
        "stabilized" => Ok(ScenarioParams::stabilized_world()),
        other => anyhow::bail!("Unknown preset '{}'. Use: bau, technology, stabilized", other),
    }
}

fn print_summary(sim: &SimulationOutput) {
    println!(
        "{:>6}  {:>12}  {:>10}  {:>10}  {:>8}  {:>8}",
        "Year", "Population", "Food/cap", "Ind.Out/cap", "NNR%", "PollIdx"
    );
    println!("{}", "-".repeat(64));

    for state in sim.states.iter().step_by(10) {
        println!(
            "{:>6.0}  {:>12.2e}  {:>10.1}  {:>10.1}  {:>8.1}  {:>8.2}",
            state.time,
            state.population.population,
            state.agriculture.food_per_capita,
            state.capital.industrial_output_per_capita,
            state.resources.fraction_remaining * 100.0,
            state.pollution.pollution_index,
        );
    }
}

fn write_csv(sim: &SimulationOutput, path: &PathBuf) -> Result<()> {
    let mut wtr = csv::Writer::from_path(path)
        .with_context(|| format!("Cannot write to {}", path.display()))?;

    wtr.write_record([
        "year",
        "population",
        "cohort_0_14",
        "cohort_15_44",
        "cohort_45_64",
        "cohort_65_plus",
        "birth_rate",
        "death_rate",
        "life_expectancy",
        "fertility_rate",
        "industrial_capital",
        "service_capital",
        "industrial_output",
        "industrial_output_per_capita",
        "service_output_per_capita",
        "arable_land",
        "potentially_arable_land",
        "urban_industrial_land",
        "land_fertility",
        "food",
        "food_per_capita",
        "land_yield",
        "nnr_fraction",
        "persistent_pollution",
        "pollution_index",
    ])?;

    for s in &sim.states {
        wtr.write_record(&[
            format!("{:.1}", s.time),
            format!("{:.4e}", s.population.population),
            format!("{:.4e}", s.population.cohort_0_14),
            format!("{:.4e}", s.population.cohort_15_44),
            format!("{:.4e}", s.population.cohort_45_64),
            format!("{:.4e}", s.population.cohort_65_plus),
            format!("{:.6}", s.population.birth_rate),
            format!("{:.6}", s.population.death_rate),
            format!("{:.2}", s.population.life_expectancy),
            format!("{:.3}", s.population.fertility_rate),
            format!("{:.4e}", s.capital.industrial_capital),
            format!("{:.4e}", s.capital.service_capital),
            format!("{:.4e}", s.capital.industrial_output),
            format!("{:.2}", s.capital.industrial_output_per_capita),
            format!("{:.2}", s.capital.service_output_per_capita),
            format!("{:.4e}", s.agriculture.arable_land),
            format!("{:.4e}", s.agriculture.potentially_arable_land),
            format!("{:.4e}", s.agriculture.urban_industrial_land),
            format!("{:.1}", s.agriculture.land_fertility),
            format!("{:.4e}", s.agriculture.food),
            format!("{:.2}", s.agriculture.food_per_capita),
            format!("{:.2}", s.agriculture.land_yield),
            format!("{:.4}", s.resources.fraction_remaining),
            format!("{:.4e}", s.pollution.persistent_pollution),
            format!("{:.4}", s.pollution.pollution_index),
        ])?;
    }

    wtr.flush()?;
    Ok(())
}

/// Thin wrapper around `world3_core::validation::validate_bau()`.
/// Runs BAU simulation, prints PASS/FAIL for each check, exits 1 on failure.
fn validate() -> Result<()> {
    eprintln!("Running BAU validation against World3 reference dynamics…\n");

    let params = ScenarioParams::bau();
    let initial = WorldState::initial_1900();
    let tables = std::sync::Arc::new(world3_core::lookup::tables::WorldLookupTables::load());
    let solver = Rk4Solver::new(tables);
    let states = solver.solve(initial, &params)?;
    let sim = SimulationOutput::new(states, params);

    let results = world3_core::validation::validate_bau(&sim);

    let mut failed = false;
    for r in &results {
        let status = if r.passed { "PASS" } else { "FAIL" };
        eprintln!("  {}  {}: {}", status, r.label, r.detail);
        if !r.passed {
            failed = true;
        }
    }

    eprintln!();
    if failed {
        anyhow::bail!("Validation failed");
    } else {
        eprintln!("Validation PASSED ({} checks)", results.len());
        Ok(())
    }
}
