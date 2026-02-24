# Chart Output

The `--chart` flag on the `simulate` command renders a normalized Limits-to-Growth style PNG chart.

## Usage

```bash
# Generate chart only (summary printed to stdout)
cargo run --bin world3-cli -- simulate --preset bau --chart bau_chart.png

# Generate chart + CSV
cargo run --bin world3-cli -- simulate --preset bau --output data.csv --chart bau_chart.png

# Chart with custom time range
cargo run --bin world3-cli -- simulate --preset stabilized \
  --start 1970 --end 2100 --chart stabilized.png
```

## Chart Layout

- **Dimensions**: 1200 x 800 pixels
- **Title**: `<Scenario Name> — Normalized` (e.g. "Business as Usual — Normalized")
- **X axis**: Year (simulation range, 10 labels)
- **Y axis**: Normalized value (0.0 to 1.05, 10 labels)
- **Legend**: Upper-right corner with white background

## Series

Six variables are plotted, each normalized to [0, 1] by dividing by its maximum value over the simulation period. Resources (already in [0, 1]) are plotted directly.

| Series | Color | Hex | Normalization |
|--------|-------|-----|---------------|
| Resources | Teal | `#2a9d8f` | Already 0-1 (fraction remaining) |
| Food / capita | Gold | `#e9c46a` | Divided by max |
| Population | Brown | `#8b5e3c` | Divided by max |
| Services / cap | Steel blue | `#457b9d` | Divided by max |
| Ind. output / cap | Red | `#e63946` | Divided by max |
| Pollution | Gray | `#6c757d` | Divided by max |

## Example: BAU Standard Run

![BAU Standard Run](examples/bau_standard_run.png)

The BAU (Business as Usual) chart shows the classic Limits to Growth dynamics:

- **Resources** (teal) decline steadily as extraction continues without efficiency improvements
- **Population** (brown) peaks around 2030 at ~6.3B then declines as death rates rise from pollution and reduced services
- **Industrial output per capita** (red) peaks around 2070-2080 then begins declining as resource scarcity increases costs
- **Food per capita** (gold) remains relatively stable through most of the run, rising in the latter half as population declines faster than food production
- **Pollution** (gray) rises dramatically through the 21st century, peaking around 2090
- **Services per capita** (steel blue) tracks industrial output with a slight lag

This matches the qualitative dynamics of Meadows et al. 1972, Fig. 35.

## Technical Details

The chart is rendered using the [plotters](https://crates.io/crates/plotters) crate with `BitMapBackend`. The `render_chart()` function is defined in `crates/world3-cli/src/main.rs`.

Each series is drawn as a 2px-wide line. The normalization ensures all variables are visually comparable on a single axis despite having vastly different units and magnitudes (e.g. population in billions vs. pollution index ~0-170).
