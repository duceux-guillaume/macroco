mod diagnose;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use plotters::prelude::*;
use std::path::{Path, PathBuf};
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

        /// Output chart image (PNG) file path
        #[arg(long)]
        chart: Option<PathBuf>,
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
        Commands::Simulate { preset, output, start, end, dt, chart } => {
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

            if let Some(chart_path) = chart {
                render_chart(&sim, &chart_path)?;
                eprintln!("Wrote chart {}", chart_path.display());
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

/// Render a normalized Limits-to-Growth style chart as PNG.
fn render_chart(sim: &SimulationOutput, path: &Path) -> Result<()> {
    // Extract raw series
    let years: Vec<f64> = sim.states.iter().map(|s| s.time).collect();
    let population: Vec<f64> = sim.states.iter().map(|s| s.population.population).collect();
    let resources: Vec<f64> = sim.states.iter().map(|s| s.resources.fraction_remaining).collect();
    let food_pc: Vec<f64> = sim.states.iter().map(|s| s.agriculture.food_per_capita).collect();
    let ind_out_pc: Vec<f64> = sim
        .states
        .iter()
        .map(|s| s.capital.industrial_output_per_capita)
        .collect();
    let svc_out_pc: Vec<f64> = sim
        .states
        .iter()
        .map(|s| s.capital.service_output_per_capita)
        .collect();
    let pollution: Vec<f64> = sim.states.iter().map(|s| s.pollution.pollution_index).collect();

    // Normalize each series to 0–1 by dividing by max (resources already 0–1)
    let normalize = |v: &[f64]| -> Vec<f64> {
        let max = v.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        if max <= 0.0 {
            return vec![0.0; v.len()];
        }
        v.iter().map(|x| x / max).collect()
    };

    let series: Vec<(&str, Vec<f64>, RGBColor)> = vec![
        ("Resources", resources, RGBColor(42, 157, 143)),        // #2a9d8f
        ("Food / capita", normalize(&food_pc), RGBColor(233, 196, 106)), // #e9c46a
        ("Population", normalize(&population), RGBColor(139, 94, 60)),   // #8b5e3c
        ("Services / cap", normalize(&svc_out_pc), RGBColor(69, 123, 157)), // #457b9d
        ("Ind. output / cap", normalize(&ind_out_pc), RGBColor(230, 57, 70)), // #e63946
        ("Pollution", normalize(&pollution), RGBColor(108, 117, 125)),   // #6c757d
    ];

    let x_min = years.first().copied().unwrap_or(1900.0);
    let x_max = years.last().copied().unwrap_or(2100.0);

    let root = BitMapBackend::new(path, (1200, 800)).into_drawing_area();
    root.fill(&WHITE)?;

    let title = format!("{} — Normalized", sim.params.meta.name);
    let mut chart = ChartBuilder::on(&root)
        .caption(&title, ("sans-serif", 28).into_font())
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(x_min..x_max, 0.0..1.05)?;

    chart
        .configure_mesh()
        .x_desc("Year")
        .y_desc("Normalized value")
        .x_labels(10)
        .y_labels(10)
        .draw()?;

    for (label, data, color) in &series {
        chart
            .draw_series(LineSeries::new(
                years.iter().copied().zip(data.iter().copied()),
                ShapeStyle::from(color).stroke_width(2),
            ))?
            .label(*label)
            .legend(move |(x, y)| {
                PathElement::new(vec![(x, y), (x + 20, y)], ShapeStyle::from(color).stroke_width(2))
            });
    }

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperRight)
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    root.present()?;
    Ok(())
}

/// Validate the BAU run against Meadows 1972/2004 reference checkpoints.
///
/// Checks qualitative dynamics (not exact values) against the World3 standard run:
///   - Population: 1.6B (1900) → grows → peaks 6-12B (2000-2070) → declines
///   - NNR: monotonic depletion, significantly depleted by mid-century
///   - Pollution: rises, peaks, may decline after industrial collapse
///   - IOPC: rises, peaks, collapses before 2100
///   - Life expectancy: rises then falls with environmental degradation
fn validate() -> Result<()> {
    eprintln!("Running BAU validation against World3 reference dynamics…\n");

    let params = ScenarioParams::bau();
    let initial = WorldState::initial_1900();
    let tables = std::sync::Arc::new(world3_core::lookup::tables::WorldLookupTables::load());
    let solver = Rk4Solver::new(tables);
    let states = solver.solve(initial, &params)?;
    let sim = SimulationOutput::new(states, params);

    let mut failures: Vec<String> = Vec::new();
    let mut pass_count: usize = 0;

    // Helper to check a value within a range; returns true if passed
    let check = |label: &str, value: f64, lo: f64, hi: f64, unit: &str,
                 failures: &mut Vec<String>, pass_count: &mut usize| {
        if !(lo..=hi).contains(&value) {
            failures.push(format!("{label}: {value:.3e} {unit} outside [{lo:.3e}, {hi:.3e}]"));
            eprintln!("  FAIL  {label}: {value:.3e} {unit}");
        } else {
            *pass_count += 1;
            eprintln!("  PASS  {label}: {value:.3e} {unit}");
        }
    };

    // --- Population trajectory checkpoints ---
    eprintln!("Population trajectory:");

    {
        let s = sim.state_at_year(1900.0).expect("missing state at year 1900");
        check("  1900 population", s.population.population, 1.4e9, 1.8e9, "", &mut failures, &mut pass_count);
    }

    {
        let s = sim.state_at_year(1950.0).expect("missing state at year 1950");
        check("  1950 population", s.population.population, 2.0e9, 4.0e9, "", &mut failures, &mut pass_count);
    }

    {
        let s = sim.state_at_year(1970.0).expect("missing state at year 1970");
        check("  1970 population", s.population.population, 3.0e9, 5.5e9, "", &mut failures, &mut pass_count);
    }

    // Peak population: 6-12B somewhere in 2000-2070
    let (peak_pop, peak_year) = sim
        .states
        .iter()
        .fold((0.0_f64, 0.0_f64), |(mp, my), s| {
            if s.population.population > mp {
                (s.population.population, s.time)
            } else {
                (mp, my)
            }
        });

    if !(5.0e9..=12.0e9).contains(&peak_pop) || !(1990.0..=2070.0).contains(&peak_year) {
        failures.push(format!(
            "Population peak {:.2e} at {:.0} outside expected [5B–12B, 1990–2070]",
            peak_pop, peak_year
        ));
        eprintln!("  FAIL  Population peak: {:.2e} at year {:.0}", peak_pop, peak_year);
    } else {
        pass_count += 1;
        eprintln!("  PASS  Population peak: {:.2e} at year {:.0}", peak_pop, peak_year);
    }

    // Population at 2100 should be less than peak (decline phase)
    {
        let s = sim.state_at_year(2100.0).expect("missing state at year 2100");
        let pop_2100 = s.population.population;
        if pop_2100 >= peak_pop * 0.95 {
            failures.push(format!(
                "2100 population {:.2e} not declining (peak was {:.2e})",
                pop_2100, peak_pop
            ));
            eprintln!("  FAIL  2100 population: {:.2e} (should be < peak)", pop_2100);
        } else {
            pass_count += 1;
            eprintln!("  PASS  2100 population: {:.2e} (declining from peak)", pop_2100);
        }
    }

    // --- Resource depletion ---
    eprintln!("\nResource depletion:");

    {
        let s = sim.state_at_year(2000.0).expect("missing state at year 2000");
        check("  2000 NNR fraction", s.resources.fraction_remaining, 0.0, 0.50, "", &mut failures, &mut pass_count);
    }

    {
        let s = sim.state_at_year(2100.0).expect("missing state at year 2100");
        check("  2100 NNR fraction", s.resources.fraction_remaining, 0.0, 0.30, "", &mut failures, &mut pass_count);
    }

    // NNR should decrease monotonically (check every 20 years)
    let nnr_monotonic = [1920.0, 1940.0, 1960.0, 1980.0, 2000.0, 2020.0, 2040.0, 2060.0, 2080.0, 2100.0]
        .windows(2)
        .all(|pair| {
            let a = sim.state_at_year(pair[0]).expect("missing NNR state").resources.fraction_remaining;
            let b = sim.state_at_year(pair[1]).expect("missing NNR state").resources.fraction_remaining;
            b <= a + 0.001 // allow tiny numerical noise
        });
    if !nnr_monotonic {
        failures.push("NNR fraction not monotonically decreasing".into());
        eprintln!("  FAIL  NNR monotonically decreasing");
    } else {
        pass_count += 1;
        eprintln!("  PASS  NNR monotonically decreasing");
    }

    // --- Pollution ---
    eprintln!("\nPollution:");

    let max_pollution = sim
        .states
        .iter()
        .map(|s| s.pollution.pollution_index)
        .fold(0.0_f64, f64::max);
    check("  Peak pollution index", max_pollution, 1.0, 100.0, "", &mut failures, &mut pass_count);

    // --- Industrial output ---
    eprintln!("\nIndustrial dynamics:");

    // IOPC should peak and then collapse
    let (peak_iopc, peak_iopc_year) = sim
        .states
        .iter()
        .fold((0.0_f64, 0.0_f64), |(mp, my), s| {
            if s.capital.industrial_output_per_capita > mp {
                (s.capital.industrial_output_per_capita, s.time)
            } else {
                (mp, my)
            }
        });

    {
        let s = sim.state_at_year(2100.0).expect("missing state at year 2100");
        let iopc_2100 = s.capital.industrial_output_per_capita;
        if iopc_2100 > peak_iopc * 0.5 {
            failures.push(format!(
                "IOPC at 2100 ({:.0}) not collapsed (peak {:.0} at {:.0})",
                iopc_2100, peak_iopc, peak_iopc_year
            ));
            eprintln!("  FAIL  IOPC collapse: {:.0} at 2100 (peak {:.0} at {:.0})", iopc_2100, peak_iopc, peak_iopc_year);
        } else {
            pass_count += 1;
            eprintln!("  PASS  IOPC collapse: {:.0} at 2100 (peak {:.0} at {:.0})", iopc_2100, peak_iopc, peak_iopc_year);
        }
    }

    // --- Life expectancy ---
    eprintln!("\nLife expectancy:");

    // LE should rise from ~30 in 1900 to 50+ mid-century, then fall
    let (peak_le, peak_le_year) = sim
        .states
        .iter()
        .filter(|s| s.time >= 1910.0) // skip year 0 (initial state before recomputation)
        .fold((0.0_f64, 0.0_f64), |(mp, my), s| {
            if s.population.life_expectancy > mp {
                (s.population.life_expectancy, s.time)
            } else {
                (mp, my)
            }
        });

    check("  Peak LE", peak_le, 45.0, 80.0, "yr", &mut failures, &mut pass_count);

    {
        let s = sim.state_at_year(2100.0).expect("missing state at year 2100");
        let le_2100 = s.population.life_expectancy;
        if le_2100 > peak_le * 0.8 {
            failures.push(format!("LE at 2100 ({:.1}) not declining (peak {:.1} at {:.0})", le_2100, peak_le, peak_le_year));
            eprintln!("  FAIL  LE decline: {:.1} at 2100 (peak {:.1} at {:.0})", le_2100, peak_le, peak_le_year);
        } else {
            pass_count += 1;
            eprintln!("  PASS  LE decline: {:.1} at 2100 (peak {:.1} at {:.0})", le_2100, peak_le, peak_le_year);
        }
    }

    // --- Summary ---
    let total_checks = pass_count + failures.len();
    eprintln!();
    if failures.is_empty() {
        eprintln!("Validation PASSED — qualitative dynamics match World3 standard run.");
        eprintln!("({} checks passed)", total_checks);
        Ok(())
    } else {
        eprintln!("Validation FAILED:");
        for f in &failures {
            eprintln!("  FAIL  {}", f);
        }
        anyhow::bail!("Validation failed with {} issue(s)", failures.len());
    }
}
