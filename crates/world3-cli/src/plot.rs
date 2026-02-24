use anyhow::Result;
use plotters::prelude::*;
use world3_core::output::SimulationOutput;

pub struct SeriesConfig {
    pub label: &'static str,
    pub field: &'static str,
    pub color: RGBColor,
    pub scale: f64,
}

pub struct PanelConfig {
    pub title: &'static str,
    pub y_label: &'static str,
    pub series: &'static [SeriesConfig],
}

pub struct ChartConfig {
    pub title: &'static str,
    pub width: u32,
    pub height_per_panel: u32,
    pub panels: &'static [PanelConfig],
}

static POPULATION_SERIES: &[SeriesConfig] = &[
    SeriesConfig {
        label: "Population (B)",
        field: "population.population",
        color: RGBColor(31, 119, 180),
        scale: 1e-9,
    },
    SeriesConfig {
        label: "Life Expectancy (yr)",
        field: "population.life_expectancy",
        color: RGBColor(255, 127, 14),
        scale: 1.0,
    },
];

static CAPITAL_SERIES: &[SeriesConfig] = &[
    SeriesConfig {
        label: "Ind. Output/cap ($/yr)",
        field: "capital.industrial_output_per_capita",
        color: RGBColor(44, 160, 44),
        scale: 1.0,
    },
    SeriesConfig {
        label: "Svc. Output/cap ($/yr)",
        field: "capital.service_output_per_capita",
        color: RGBColor(214, 39, 40),
        scale: 1.0,
    },
];

static AGRICULTURE_SERIES: &[SeriesConfig] = &[
    SeriesConfig {
        label: "Food/cap (kg/yr)",
        field: "agriculture.food_per_capita",
        color: RGBColor(148, 103, 189),
        scale: 1.0,
    },
    SeriesConfig {
        label: "Land Yield (kg/ha)",
        field: "agriculture.land_yield",
        color: RGBColor(140, 86, 75),
        scale: 1.0,
    },
];

static RESOURCES_SERIES: &[SeriesConfig] = &[SeriesConfig {
    label: "NNR Fraction Remaining",
    field: "resources.fraction_remaining",
    color: RGBColor(227, 119, 194),
    scale: 1.0,
}];

static POLLUTION_SERIES: &[SeriesConfig] = &[SeriesConfig {
    label: "Pollution Index",
    field: "pollution.pollution_index",
    color: RGBColor(127, 127, 127),
    scale: 1.0,
}];

static DEFAULT_PANELS: &[PanelConfig] = &[
    PanelConfig {
        title: "Population",
        y_label: "Value",
        series: POPULATION_SERIES,
    },
    PanelConfig {
        title: "Capital",
        y_label: "$/yr per capita",
        series: CAPITAL_SERIES,
    },
    PanelConfig {
        title: "Agriculture",
        y_label: "Value",
        series: AGRICULTURE_SERIES,
    },
    PanelConfig {
        title: "Non-Renewable Resources",
        y_label: "Fraction",
        series: RESOURCES_SERIES,
    },
    PanelConfig {
        title: "Pollution",
        y_label: "Index",
        series: POLLUTION_SERIES,
    },
];

static DEFAULT_CHART_CONFIG: ChartConfig = ChartConfig {
    title: "World 3 Simulation",
    width: 1200,
    height_per_panel: 220,
    panels: DEFAULT_PANELS,
};

pub fn default_chart_config() -> &'static ChartConfig {
    &DEFAULT_CHART_CONFIG
}

pub fn render_chart(
    sim: &SimulationOutput,
    path: &std::path::Path,
    config: &ChartConfig,
) -> Result<()> {
    let n_panels = config.panels.len() as u32;
    let total_height = config.height_per_panel * n_panels + 80;

    let root = BitMapBackend::new(path, (config.width, total_height)).into_drawing_area();
    root.fill(&WHITE)?;

    // Draw chart title in the header strip (top 80px)
    let (header, body) = root.split_vertically(80);
    header.fill(&WHITE)?;
    header.draw_text(
        config.title,
        &TextStyle::from(("sans-serif", 28).into_font()).color(&BLACK),
        (config.width as i32 / 2 - 120, 20),
    )?;

    // Split body evenly for each panel
    let panels_areas = body.split_evenly((n_panels as usize, 1));

    let x_range = {
        let first = sim.timeline.first().copied().unwrap_or(1900.0);
        let last = sim.timeline.last().copied().unwrap_or(2100.0);
        first..last
    };

    for (panel_cfg, area) in config.panels.iter().zip(panels_areas.iter()) {
        // Collect all series data
        let mut all_series: Vec<(Vec<f64>, &SeriesConfig)> = Vec::new();
        for sc in panel_cfg.series {
            let raw = sim.extract_series(sc.field);
            let all_nan = raw.iter().all(|v| v.is_nan());
            if all_nan {
                eprintln!(
                    "warn: series '{}' produced all-NaN values — check field path",
                    sc.field
                );
            }
            let scaled: Vec<f64> = raw.iter().map(|v| v * sc.scale).collect();
            all_series.push((scaled, sc));
        }

        // Compute y-range across all series
        let (y_min, y_max) = all_series.iter().fold(
            (f64::INFINITY, f64::NEG_INFINITY),
            |(mn, mx), (vals, _)| {
                vals.iter().filter(|v| v.is_finite()).fold((mn, mx), |(mn, mx), &v| {
                    (mn.min(v), mx.max(v))
                })
            },
        );

        let span = (y_max - y_min).max(1.0);
        let margin = span * 0.05;
        let y_range = (y_min - margin)..(y_max + margin);

        let mut chart = ChartBuilder::on(area)
            .caption(panel_cfg.title, ("sans-serif", 16).into_font())
            .margin(8)
            .x_label_area_size(30)
            .y_label_area_size(60)
            .build_cartesian_2d(x_range.clone(), y_range)?;

        chart
            .configure_mesh()
            .x_desc("Year")
            .y_desc(panel_cfg.y_label)
            .x_label_formatter(&|v| format!("{:.0}", v))
            .draw()?;

        for (vals, sc) in &all_series {
            let data: Vec<(f64, f64)> = sim
                .timeline
                .iter()
                .zip(vals.iter())
                .filter(|(_, v)| v.is_finite())
                .map(|(&t, &v)| (t, v))
                .collect();

            chart
                .draw_series(LineSeries::new(data, sc.color.stroke_width(2)))?
                .label(sc.label)
                .legend(move |(x, y)| {
                    PathElement::new(vec![(x, y), (x + 20, y)], sc.color.stroke_width(2))
                });
        }

        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()?;
    }

    root.present()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn run_short_bau() -> SimulationOutput {
        let mut params = ScenarioParams::bau();
        params.start_year = 1900.0;
        params.end_year = 1950.0;
        params.time_step = 1.0;

        let initial = WorldState {
            time: 1900.0,
            population: PopulationState {
                population: 1.6e9,
                cohort_0_14: 0.60e9,
                cohort_15_44: 0.65e9,
                cohort_45_64: 0.27e9,
                cohort_65_plus: 0.08e9,
                ..Default::default()
            },
            capital: CapitalState {
                industrial_capital: 0.2e12,
                service_capital: 0.32e12,
                ..Default::default()
            },
            agriculture: AgricultureState {
                arable_land: 0.9e9,
                potentially_arable_land: 2.3e9,
                food_per_capita: 400.0,
                ..Default::default()
            },
            resources: ResourceState {
                nonrenewable_resources: 1.0,
                fraction_remaining: 1.0,
            },
            pollution: PollutionState {
                persistent_pollution: 0.05,
                pollution_index: 0.05,
                ..Default::default()
            },
        };

        let tables = std::sync::Arc::new(world3_core::lookup::tables::WorldLookupTables::load());
        let solver = Rk4Solver::new(tables);
        let states = solver.solve(initial, &params).expect("simulation failed");
        SimulationOutput::new(states, params)
    }

    #[test]
    fn render_chart_creates_valid_png() {
        let sim = run_short_bau();
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("test_chart.png");

        render_chart(&sim, &path, default_chart_config()).expect("render_chart failed");

        // Verify PNG magic bytes: 0x89 0x50 0x4E 0x47
        let bytes = std::fs::read(&path).expect("read png");
        assert!(
            bytes.len() > 8,
            "PNG file is too small: {} bytes",
            bytes.len()
        );
        assert_eq!(
            &bytes[..4],
            &[0x89, 0x50, 0x4E, 0x47],
            "File does not start with PNG magic bytes"
        );
    }
}
