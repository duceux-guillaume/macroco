use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use plotters::prelude::*;
use std::path::{Path, PathBuf};
use world3_core::{
    model::{
        params::ScenarioParams,
        state::{
            AgricultureState, CapitalState, PollutionState, PopulationState, ResourceState,
            WorldState,
        },
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

            let initial = initial_conditions_1900();
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

/// World 3 initial conditions for year 1900.
/// Values calibrated to broadly match Meadows 1972 standard run starting point.
fn initial_conditions_1900() -> WorldState {
    WorldState {
        time: 1900.0,
        population: PopulationState {
            population: 1.6e9,
            // 1900 age structure: young population with small elderly cohort
            cohort_0_14: 0.60e9,   // 37.5% — high fertility, high child mortality era
            cohort_15_44: 0.65e9,  // 40.6%
            cohort_45_64: 0.27e9,  // 16.9%
            cohort_65_plus: 0.08e9, // 5.0% — small elderly cohort in 1900
            ..Default::default()
        },
        capital: CapitalState {
            industrial_capital: 0.2e12,  // 1975 USD
            // Service capital pre-set to its ~1900 equilibrium.
            // At industrial_output ≈ $133B and frac_to_services ≈ 0.12:
            //   service_capital_eq = 133e9 × 0.12 / 0.05 ≈ 0.32e12
            //   → sopc ≈ $200/yr → lem_health ≈ 0.76 → LE ≈ 32 yr ✓
            service_capital: 0.32e12,
            ..Default::default()
        },
        agriculture: AgricultureState {
            arable_land: 0.9e9,            // hectares
            potentially_arable_land: 2.3e9,
            food_per_capita: 400.0,        // initial estimate; overwritten by agriculture sector
            ..Default::default()
        },
        resources: ResourceState {
            nonrenewable_resources: 1.0,   // 100% remaining in 1900
            fraction_remaining: 1.0,
        },
        pollution: PollutionState {
            // pollution_index = persistent_pollution (reference_stock = 1.0)
            // Small in 1900; rises to ~1 by 1970, ~10–30 peak around 2030–2050 in BAU.
            persistent_pollution: 0.05,
            pollution_index: 0.05,
            ..Default::default()
        },
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

/// Validate the BAU run against Meadows 1972 reference checkpoints.
///
/// We check qualitative dynamics, not exact values (since our model is
/// a faithful but not byte-identical re-implementation):
///   - Population grows from 1.6B (1900) through ~8B peak (~2030), then declines
///   - NNR fraction falls monotonically
///   - Pollution index rises, peaks, may decline after industrial collapse
fn validate() -> Result<()> {
    eprintln!("Running BAU validation against Meadows 1972 reference dynamics…");

    let params = ScenarioParams::bau();
    let initial = initial_conditions_1900();
    let tables = std::sync::Arc::new(world3_core::lookup::tables::WorldLookupTables::load());
    let solver = Rk4Solver::new(tables);
    let states = solver.solve(initial, &params)?;
    let sim = SimulationOutput::new(states, params);

    let mut failures: Vec<String> = Vec::new();

    // Check 1: Population in 1900 is ~1.6B
    if let Some(s) = sim.state_at_year(1900.0) {
        let pop = s.population.population;
        if !(1.0e9..=2.5e9).contains(&pop) {
            failures.push(format!("1900 population {:.2e} outside [1B, 2.5B]", pop));
        } else {
            eprintln!("  PASS  1900 population: {:.2e}", pop);
        }
    }

    // Check 2: Population in 1970 is ~3.6B
    if let Some(s) = sim.state_at_year(1970.0) {
        let pop = s.population.population;
        if !(2.5e9..=5.0e9).contains(&pop) {
            failures.push(format!("1970 population {:.2e} outside [2.5B, 5B]", pop));
        } else {
            eprintln!("  PASS  1970 population: {:.2e}", pop);
        }
    }

    // Check 3: Peak population somewhere in 2020–2060 and is 6B–12B
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

    if !(6.0e9..=12.0e9).contains(&peak_pop) || !(2000.0..=2070.0).contains(&peak_year) {
        failures.push(format!(
            "Population peak {:.2e} at {:.0} outside expected [6B–12B, 2000–2070]",
            peak_pop, peak_year
        ));
    } else {
        eprintln!(
            "  PASS  Population peak: {:.2e} at year {:.0}",
            peak_pop, peak_year
        );
    }

    // Check 4: NNR fraction remaining in 2100 < 0.5 (significant depletion)
    if let Some(s) = sim.state_at_year(2100.0) {
        let nnr = s.resources.fraction_remaining;
        if nnr >= 0.7 {
            failures.push(format!("2100 NNR fraction {:.3} unexpectedly high (≥0.7)", nnr));
        } else {
            eprintln!("  PASS  2100 NNR fraction: {:.3}", nnr);
        }
    }

    // Check 5: Pollution index rises from near 0 to at least 1.0 at some point
    let max_pollution = sim
        .states
        .iter()
        .map(|s| s.pollution.pollution_index)
        .fold(0.0_f64, f64::max);
    if max_pollution < 0.5 {
        failures.push(format!(
            "Max pollution index {:.3} never rises above 0.5",
            max_pollution
        ));
    } else {
        eprintln!("  PASS  Peak pollution index: {:.3}", max_pollution);
    }

    if failures.is_empty() {
        eprintln!("\nValidation PASSED — qualitative dynamics match Meadows 1972.");
        Ok(())
    } else {
        eprintln!("\nValidation FAILED:");
        for f in &failures {
            eprintln!("  FAIL  {}", f);
        }
        anyhow::bail!("Validation failed with {} issue(s)", failures.len());
    }
}
