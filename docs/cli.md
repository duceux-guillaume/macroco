# CLI Reference

The `world3-cli` binary provides batch simulation, validation, and chart generation.

## Build & Install

```bash
cargo build --release --bin world3-cli

# Or run directly via cargo
cargo run --bin world3-cli -- <SUBCOMMAND> [OPTIONS]
```

## Commands

### `simulate`

Run a simulation and output results.

```bash
cargo run --bin world3-cli -- simulate [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--preset <NAME>` | `bau` | Scenario preset: `bau`, `technology`, `stabilized` |
| `--output <FILE>` | _(none)_ | Write results to CSV file |
| `--start <YEAR>` | `1900` | Simulation start year |
| `--end <YEAR>` | `2100` | Simulation end year |
| `--dt <YEARS>` | `1.0` | Time step in years |
| `--chart <FILE>` | _(none)_ | Render a normalized PNG chart |

**Output formats:**

- **No flags**: prints a summary table to stdout (every 10th year)
- **`--output`**: writes a 22-column CSV with all state variables
- **`--chart`**: renders a 1200x800 PNG chart with normalized key variables
- **`--output` + `--chart`**: both CSV and chart are produced

**Examples:**

```bash
# Summary table to stdout
cargo run --bin world3-cli -- simulate

# Full CSV output
cargo run --bin world3-cli -- simulate --preset bau --output output.csv

# Generate chart
cargo run --bin world3-cli -- simulate --preset bau --chart bau_chart.png

# CSV + chart + custom time range
cargo run --bin world3-cli -- simulate --preset stabilized \
  --start 1970 --end 2100 --dt 0.5 \
  --output results.csv --chart results.png
```

**CSV columns (22 fields):**

```
year, population, cohort_0_14, cohort_15_44, cohort_45_64, cohort_65_plus,
birth_rate, death_rate, life_expectancy, fertility_rate,
industrial_capital, service_capital, industrial_output, industrial_output_per_capita,
service_output_per_capita, arable_land, food, food_per_capita, land_yield,
nnr_fraction, persistent_pollution, pollution_index
```

### `validate`

Validate the BAU run against Meadows 1972 reference checkpoints. Checks qualitative dynamics (not exact values):

1. 1900 population in [1B, 2.5B]
2. 1970 population in [2.5B, 5B]
3. Population peaks at 6B-12B between 2000-2070
4. NNR fraction remaining in 2100 < 0.7
5. Peak pollution index > 0.5

```bash
cargo run --bin world3-cli -- validate
```

Each checkpoint prints `PASS` or `FAIL`. The command exits with code 1 if any check fails.

### `presets`

List all available scenario presets.

```bash
cargo run --bin world3-cli -- presets
```

Output:

```
Available presets:
  bau          Business as Usual (original World 3 standard run)
  technology   Comprehensive Technology scenario
  stabilized   Stabilized World scenario
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (unknown preset, solver divergence, I/O failure, validation failure) |
