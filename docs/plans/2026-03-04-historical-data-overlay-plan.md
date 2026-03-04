# Historical Data Overlay Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add real-world historical data overlays to all 6 D3 chart variables so users can compare simulation predictions against what actually happened.

**Architecture:** Pre-bundled CSVs in `data/historical/` → Backend `GET /api/v1/historical` endpoint → Frontend Svelte store → Dashed D3 lines on existing charts. No new crate dependencies needed (`csv = "1"` already in workspace).

**Tech Stack:** Rust/Axum backend, SvelteKit/TypeScript/D3 frontend, CSV data files.

**Design doc:** `docs/plans/2026-03-04-historical-data-overlay-design.md`

---

### Task 1: Create historical CSV data files

**Files:**
- Create: `data/historical/population.csv`
- Create: `data/historical/resources.csv`
- Create: `data/historical/food.csv`
- Create: `data/historical/industrial.csv`
- Create: `data/historical/pollution.csv`
- Create: `data/historical/life-expectancy.csv`

Each CSV follows this format (comment headers for provenance, then `year,value`):

```csv
# source: <source name and indicator>
# url: <download URL>
# units: <World 3 model units>
# transformation: <formula or "none">
# retrieved: 2026-03-04
year,value
```

**Step 1: Create `data/historical/` directory**

```bash
mkdir -p data/historical
```

**Step 2: Create population.csv**

Source: World Bank `SP.POP.TOTL` (World aggregate `1W`). No transformation needed — units are persons.
Download: `https://api.worldbank.org/v2/country/1W/indicator/SP.POP.TOTL?date=1960:2023&format=json&per_page=200`

Create the file with data from 1960-2023 (one row per year). Include the provenance header:
```
# source: World Bank SP.POP.TOTL
# url: https://data.worldbank.org/indicator/SP.POP.TOTL?locations=1W
# units: persons
# transformation: none
# retrieved: 2026-03-04
```

**Step 3: Create resources.csv**

Source: Our World in Data Energy Dataset (`owid-energy-data.csv`, filter `country == "World"`).
Download: `https://raw.githubusercontent.com/owid/energy-data/master/owid-energy-data.csv`

Transformation:
1. Sum `oil_production + coal_production + gas_production` (TWh/yr) per year
2. Compute cumulative sum from 1900 to each year
3. Divide by URR estimate (10,000,000 TWh ≈ 36,000 EJ)
4. `fraction_remaining = max(0, 1.0 - cumulative / 10000000)`

```
# source: Our World in Data Energy Dataset (Energy Institute / BP Statistical Review)
# url: https://github.com/owid/energy-data
# units: fraction remaining (0-1, 1900=1.0)
# transformation: max(0, 1.0 - cumulative_fossil_production_twh / 10,000,000)
# retrieved: 2026-03-04
```

**Step 4: Create food.csv**

Source: FAOSTAT Food Balance Sheets — Grand Total food supply quantity.
Download: `https://www.fao.org/faostat/en/#data/FBS` (select: World, Grand Total, Food supply quantity kg/capita/yr)

No arithmetic transformation — FAOSTAT units (kg/capita/yr) match World 3.

```
# source: FAOSTAT Food Balance Sheets - Grand Total food supply quantity
# url: https://www.fao.org/faostat/en/#data/FBS
# units: kg/person/year
# transformation: kcal_per_capita_per_day * 365 / 1200 (weighted average food energy density kcal/kg)
# retrieved: 2026-03-04
```

**Step 5: Create industrial.csv**

Source: World Bank `NV.IND.TOTL.KD` (constant 2015 USD) divided by `SP.POP.TOTL`, then deflated.
Download: `https://api.worldbank.org/v2/country/1W/indicator/NV.IND.TOTL.KD?date=1960:2023&format=json&per_page=200`

Transformation: `value_1975usd = (NV.IND.TOTL.KD / SP.POP.TOTL) / 3.51`
(3.51 = US GDP deflator ratio 2015/1975, from World Bank `NY.GDP.DEFL.ZS.AD`)

```
# source: World Bank NV.IND.TOTL.KD / SP.POP.TOTL, deflated to 1975 USD
# url: https://data.worldbank.org/indicator/NV.IND.TOTL.KD
# units: 1975 USD/person/year
# transformation: (industry_value_added_2015usd / population) / 3.51
# retrieved: 2026-03-04
```

**Step 6: Create pollution.csv**

Source: NOAA Mauna Loa CO2 annual mean.
Download: `https://gml.noaa.gov/webdata/ccgg/trends/co2/co2_annmean_mlo.txt`

Transformation: `(co2_ppm - 280) / (325.68 - 280)` where 280 = pre-industrial, 325.68 = 1970 value.

```
# source: NOAA GML Mauna Loa CO2 annual mean
# url: https://gml.noaa.gov/webdata/ccgg/trends/co2/co2_annmean_mlo.txt
# units: index (1970=1.0)
# transformation: (co2_ppm - 280) / (325.7 - 280)
# retrieved: 2026-03-04
```

**Step 7: Create life-expectancy.csv**

Source: World Bank `SP.DYN.LE00.IN` (World aggregate). No transformation.
Download: `https://api.worldbank.org/v2/country/1W/indicator/SP.DYN.LE00.IN?date=1960:2023&format=json&per_page=200`

```
# source: World Bank SP.DYN.LE00.IN
# url: https://data.worldbank.org/indicator/SP.DYN.LE00.IN?locations=1W
# units: years
# transformation: none
# retrieved: 2026-03-04
```

**Step 8: Commit**

```bash
git add data/historical/
git commit -m "data: add historical CSV files for 6 World 3 variables

Sources: World Bank, OWID Energy, FAOSTAT, NOAA Mauna Loa.
See comment headers in each CSV for provenance and transformations."
```

---

### Task 2: Backend — Historical data types and CSV parser

**Files:**
- Create: `crates/world3-api/src/historical.rs`
- Modify: `crates/world3-api/src/main.rs:1-4` (add `mod historical`)
- Modify: `crates/world3-api/Cargo.toml:12-14` (add `csv` dependency)

**Step 1: Write the unit tests for CSV parsing**

Create `crates/world3-api/src/historical.rs` with tests at the bottom:

```rust
use std::collections::HashMap;
use serde::Serialize;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct HistoricalDataPoint {
    pub year: f64,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct HistoricalVariable {
    pub variable: String,
    pub source: String,
    pub units: String,
    pub transformation: String,
    pub data: Vec<HistoricalDataPoint>,
}

// ---------------------------------------------------------------------------
// CSV parsing
// ---------------------------------------------------------------------------

/// Parse a historical CSV with comment-header metadata.
/// Lines starting with `#` are metadata; first non-comment line is the CSV header.
pub fn parse_historical_csv(variable_id: &str, content: &str) -> HistoricalVariable {
    let mut source = String::new();
    let mut units = String::new();
    let mut transformation = String::new();
    let mut data = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("# source: ") {
            source = rest.to_string();
        } else if let Some(rest) = trimmed.strip_prefix("# units: ") {
            units = rest.to_string();
        } else if let Some(rest) = trimmed.strip_prefix("# transformation: ") {
            transformation = rest.to_string();
        } else if trimmed.starts_with('#') || trimmed.starts_with("year,") {
            // Skip other comments and header row
            continue;
        } else {
            // Data row: year,value
            let parts: Vec<&str> = trimmed.splitn(2, ',').collect();
            if parts.len() == 2 {
                if let (Ok(year), Ok(value)) = (parts[0].trim().parse::<f64>(), parts[1].trim().parse::<f64>()) {
                    data.push(HistoricalDataPoint { year, value });
                }
            }
        }
    }

    HistoricalVariable {
        variable: variable_id.to_string(),
        source,
        units,
        transformation,
        data,
    }
}

/// Load all historical CSVs from a directory.
/// Returns a map of variable_id → HistoricalVariable.
/// Files are named `{variable_id}.csv` (e.g., `population.csv`).
pub fn load_historical_data(dir: &std::path::Path) -> HashMap<String, HistoricalVariable> {
    let mut map = HashMap::new();

    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            tracing::warn!("Could not read historical data dir {}: {}", dir.display(), e);
            return map;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("csv") {
            continue;
        }
        let variable_id = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };

        match std::fs::read_to_string(&path) {
            Ok(content) => {
                let var_data = parse_historical_csv(&variable_id, &content);
                tracing::info!(
                    "Loaded historical data for '{}': {} points ({} → {})",
                    variable_id,
                    var_data.data.len(),
                    var_data.data.first().map(|d| d.year).unwrap_or(0.0),
                    var_data.data.last().map(|d| d.year).unwrap_or(0.0),
                );
                map.insert(variable_id, var_data);
            }
            Err(e) => {
                tracing::warn!("Failed to read {}: {}", path.display(), e);
            }
        }
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_CSV: &str = r#"# source: World Bank SP.POP.TOTL
# url: https://data.worldbank.org/indicator/SP.POP.TOTL
# units: persons
# transformation: none
# retrieved: 2026-03-04
year,value
1960,3.034e9
1970,3.700e9
1980,4.434e9
"#;

    #[test]
    fn parse_extracts_metadata() {
        let result = parse_historical_csv("population", SAMPLE_CSV);
        assert_eq!(result.variable, "population");
        assert_eq!(result.source, "World Bank SP.POP.TOTL");
        assert_eq!(result.units, "persons");
        assert_eq!(result.transformation, "none");
    }

    #[test]
    fn parse_extracts_data_points() {
        let result = parse_historical_csv("population", SAMPLE_CSV);
        assert_eq!(result.data.len(), 3);
        assert_eq!(result.data[0].year, 1960.0);
        assert!((result.data[0].value - 3.034e9).abs() < 1.0);
        assert_eq!(result.data[1].year, 1970.0);
        assert!((result.data[1].value - 3.700e9).abs() < 1.0);
        assert_eq!(result.data[2].year, 1980.0);
    }

    #[test]
    fn parse_handles_empty_content() {
        let result = parse_historical_csv("empty", "");
        assert_eq!(result.data.len(), 0);
        assert_eq!(result.source, "");
    }

    #[test]
    fn parse_skips_malformed_lines() {
        let csv = "# source: test\nyear,value\n1960,3.034e9\nbad line\n1970,abc\n1980,4.434e9\n";
        let result = parse_historical_csv("test", csv);
        assert_eq!(result.data.len(), 2); // Only 1960 and 1980
    }

    #[test]
    fn load_returns_empty_for_missing_dir() {
        let result = load_historical_data(std::path::Path::new("/nonexistent/path"));
        assert!(result.is_empty());
    }
}
```

**Step 2: Add `mod historical` to main.rs**

In `crates/world3-api/src/main.rs`, after line 4 (`mod state;`), add:

```rust
pub mod historical;
```

**Step 3: Add `csv` dependency to world3-api Cargo.toml**

In `crates/world3-api/Cargo.toml`, after line 14 (`world3-ingestion = ...`), add:

```toml
csv = { workspace = true }
```

Note: The `csv` crate is not actually used by the parser (we hand-parse for simplicity), but it's already a workspace dependency and may be useful for future robustness. Skip this step if you prefer — the parser works without it.

**Step 4: Run tests to verify they pass**

```bash
cargo test --package world3-api -- historical::tests
```

Expected: All 5 tests pass.

**Step 5: Commit**

```bash
git add crates/world3-api/src/historical.rs crates/world3-api/src/main.rs
git commit -m "feat(api): add historical data types and CSV parser with tests"
```

---

### Task 3: Backend — Historical API endpoint and state integration

**Files:**
- Create: `crates/world3-api/src/routes/historical.rs`
- Modify: `crates/world3-api/src/routes/mod.rs:16-19` (add module declaration)
- Modify: `crates/world3-api/src/routes/mod.rs:33-44` (add routes)
- Modify: `crates/world3-api/src/state.rs:10-15` (add field to AppState)
- Modify: `crates/world3-api/src/state.rs:17-49` (load data in init)

**Step 1: Create route handler**

Create `crates/world3-api/src/routes/historical.rs`:

```rust
use std::sync::Arc;

use axum::{extract::State, Json};

use crate::{error::ApiError, historical::HistoricalVariable, state::AppState};

/// GET /api/v1/historical — return all historical variables.
pub async fn list_all(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<HistoricalVariable>> {
    let mut vars: Vec<HistoricalVariable> = state.historical.values().cloned().collect();
    vars.sort_by(|a, b| a.variable.cmp(&b.variable));
    Json(vars)
}

/// GET /api/v1/historical/:variable_id — return one historical variable.
pub async fn get_variable(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(variable_id): axum::extract::Path<String>,
) -> Result<Json<HistoricalVariable>, ApiError> {
    state
        .historical
        .get(&variable_id)
        .cloned()
        .map(Json)
        .ok_or_else(|| {
            ApiError::NotFound(format!("Historical variable '{}' not found", variable_id))
        })
}
```

**Step 2: Register module and routes in mod.rs**

In `crates/world3-api/src/routes/mod.rs`:

After line 19 (`pub mod ws;`), add:
```rust
mod historical;
```

After line 44 (the WebSocket route `.route("/ws", get(ws::ws_handler))`), add:
```rust
        // Historical data
        .route("/historical", get(historical::list_all))
        .route("/historical/:variable_id", get(historical::get_variable))
```

**Step 3: Add historical data to AppState**

In `crates/world3-api/src/state.rs`:

Add import at top (after line 1):
```rust
use crate::historical::{self, HistoricalVariable};
```

Add field to `AppState` struct (after line 12, the `scenarios` field):
```rust
    pub historical: HashMap<String, HistoricalVariable>,
```

Add loading in `init_app_state()` (after line 40, before the broadcast channel):
```rust
    // 3. Load historical data from CSV files
    let historical_dir = std::env::var("HISTORICAL_DATA_DIR")
        .unwrap_or_else(|_| "./data/historical".into());
    let historical = historical::load_historical_data(std::path::Path::new(&historical_dir));
    tracing::info!("Loaded historical data for {} variables", historical.len());
```

Update the `AppState` construction (add `historical` field):
```rust
    AppState {
        solver,
        scenarios: Arc::new(RwLock::new(map)),
        historical,
        _ingestion_tx: tx,
    }
```

**Step 4: Verify build compiles**

```bash
cargo build --package world3-api
```

**Step 5: Run all backend tests**

```bash
cargo test --workspace
```

**Step 6: Commit**

```bash
git add crates/world3-api/src/routes/historical.rs crates/world3-api/src/routes/mod.rs crates/world3-api/src/state.rs
git commit -m "feat(api): add GET /api/v1/historical endpoint

Loads CSV files from data/historical/ at startup into AppState.
Two endpoints: list all variables, get individual variable."
```

---

### Task 4: Frontend — Historical types, store, and API function

**Files:**
- Create: `frontend/src/lib/stores/historical.ts`
- Modify: `frontend/src/lib/api.ts:1-8` (add import), append function
- Modify: `frontend/src/lib/stores/chart-ui.ts:1-13` (add showHistorical store)

**Step 1: Write tests for the historical store**

Create `frontend/src/lib/stores/historical.test.ts`:

```typescript
import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { historicalData } from './historical';
import type { HistoricalVariable } from './historical';

beforeEach(() => {
	historicalData.set(new Map());
});

describe('historicalData store', () => {
	it('starts empty', () => {
		expect(get(historicalData).size).toBe(0);
	});

	it('stores historical variables by id', () => {
		const testVar: HistoricalVariable = {
			variable: 'population',
			source: 'World Bank',
			units: 'persons',
			transformation: 'none',
			data: [
				{ year: 1960, value: 3.034e9 },
				{ year: 1970, value: 3.700e9 }
			]
		};

		historicalData.update((m) => {
			const next = new Map(m);
			next.set('population', testVar);
			return next;
		});

		const data = get(historicalData);
		expect(data.size).toBe(1);
		expect(data.get('population')?.data).toHaveLength(2);
		expect(data.get('population')?.source).toBe('World Bank');
	});
});
```

**Step 2: Run test to verify it fails**

```bash
cd frontend && npx vitest run src/lib/stores/historical.test.ts
```

Expected: FAIL — module `./historical` not found.

**Step 3: Create the historical store**

Create `frontend/src/lib/stores/historical.ts`:

```typescript
import { writable } from 'svelte/store';

export interface HistoricalDataPoint {
	year: number;
	value: number;
}

export interface HistoricalVariable {
	variable: string;
	source: string;
	units: string;
	transformation: string;
	data: HistoricalDataPoint[];
}

/** Historical data keyed by variable ID (e.g., 'population', 'resources'). */
export const historicalData = writable<Map<string, HistoricalVariable>>(new Map());
```

**Step 4: Add showHistorical to chart-ui store**

In `frontend/src/lib/stores/chart-ui.ts`, after line 13 (the `compareVariable` export), add:

```typescript

/** Whether to show historical data overlay on charts. */
export const showHistorical = writable<boolean>(true);
```

**Step 5: Add API function**

In `frontend/src/lib/api.ts`, after line 58 (the `runScenario` function), add:

```typescript

export function getHistoricalData(): Promise<HistoricalVariable[]> {
	return apiFetch('/historical');
}
```

Add the import type at the top of `frontend/src/lib/api.ts`. After line 8, add:

```typescript
import type { HistoricalVariable } from './stores/historical';
```

**Step 6: Write test for the API function**

Add to `frontend/src/lib/api.test.ts`, after the `runScenario` describe block (line 171):

```typescript
describe('getHistoricalData', () => {
	it('calls /historical', async () => {
		const fetch = mockFetch([]);
		vi.stubGlobal('fetch', fetch);

		const { getHistoricalData } = await import('./api');
		await getHistoricalData();

		expect(fetch).toHaveBeenCalledWith(`${BASE}/historical`, expect.anything());
	});
});
```

**Step 7: Run tests to verify they pass**

```bash
cd frontend && npx vitest run src/lib/stores/historical.test.ts src/lib/api.test.ts
```

Expected: All tests pass.

**Step 8: Commit**

```bash
git add frontend/src/lib/stores/historical.ts frontend/src/lib/stores/historical.test.ts frontend/src/lib/stores/chart-ui.ts frontend/src/lib/api.ts frontend/src/lib/api.test.ts
git commit -m "feat(frontend): add historical data store, API function, and showHistorical toggle"
```

---

### Task 5: Frontend — Load historical data on app startup

**Files:**
- Modify: `frontend/src/routes/+layout.svelte:1-9` (add imports)
- Modify: `frontend/src/routes/+layout.svelte:71-118` (add fetch in onMount)

**Step 1: Add imports to +layout.svelte**

In `frontend/src/routes/+layout.svelte`, add to the imports (after line 6, the simulation import):

```typescript
	import { historicalData } from '$lib/stores/historical';
	import { getHistoricalData } from '$lib/api';
```

**Step 2: Add historical data fetch in onMount**

In `frontend/src/routes/+layout.svelte`, after line 118 (end of the `} catch (e) { console.error('Failed to load scenarios:', e); }` block), add:

```typescript

		// 3. Load historical data
		try {
			const histVars = await getHistoricalData();
			const histMap = new Map<string, typeof histVars[0]>();
			for (const v of histVars) {
				histMap.set(v.variable, v);
			}
			historicalData.set(histMap);
		} catch (e) {
			console.warn('Historical data not available:', e);
		}
```

Note: Uses `console.warn` not `console.error` — historical data is optional/enhancing, not critical.

**Step 3: Verify frontend builds**

```bash
cd frontend && npm run check
```

**Step 4: Commit**

```bash
git add frontend/src/routes/+layout.svelte
git commit -m "feat(frontend): load historical data on app startup"
```

---

### Task 6: Frontend — Historical overlay on UnifiedChart

**Files:**
- Modify: `frontend/src/lib/charts/UnifiedChart.svelte`

This is the most complex task. The UnifiedChart has two modes (normal and compare), and we need to add historical data in both.

**Step 1: Add imports**

In `frontend/src/lib/charts/UnifiedChart.svelte`, after line 9 (the `chart-ui` import), add:

```typescript
	import { showHistorical } from '../stores/chart-ui';
	import { historicalData } from '../stores/historical';
```

**Step 2: Add `historical` flag to LineDatum interface**

In `frontend/src/lib/charts/UnifiedChart.svelte`, modify the `LineDatum` interface (lines 90-97) to add a `historical` flag:

```typescript
	interface LineDatum {
		id: string;
		color: string;
		points: Array<{ year: number; normalized: number }>;
		rawPoints: Array<{ year: number; value: number }>;
		label: string;
		format: string;
		historical?: boolean;
	}
```

**Step 3: Read stores in $effect**

In the `$effect` block, after line 107 (`const _visibleVars = $visibleVariables;`), add:

```typescript
		const _showHistorical = $showHistorical;
		const _historicalData = $historicalData;
```

**Step 4: Add historical lines in normal mode**

In the `else` block (normal mode), after line 165 (closing `}` of the `for (const varConfig of unifiedVariables)` loop), add:

```typescript
			// Add historical overlay lines
			if (_showHistorical) {
				for (const varConfig of unifiedVariables) {
					if (!_visibleVars.has(varConfig.fieldPath)) continue;
					const histVar = _historicalData.get(varConfig.id);
					if (!histVar || histVar.data.length === 0) continue;

					// Normalize historical data using same range as simulation
					const simRawPoints = extractSeries(states, varConfig.fieldPath);
					const { min, max } = normalizeSeries(simRawPoints);
					const range = max - min;

					const histPoints = histVar.data.map((d) => ({
						year: d.year,
						normalized: range > 0 ? (d.value - min) / range : 0.5
					}));
					const histRawPoints = histVar.data.map((d) => ({
						year: d.year,
						value: d.value
					}));

					linesData.push({
						id: `hist-${varConfig.id}`,
						color: varConfig.color,
						points: histPoints,
						rawPoints: histRawPoints,
						label: `${varConfig.shortLabel} hist.`,
						format: varConfig.format,
						historical: true
					});
				}
			}
```

**Step 5: Add historical lines in compare mode**

In the compare mode block, after line 133 (closing `}` of the `for (const [scenarioId, states] of _data)` loop), add:

```typescript
			// Add historical overlay in compare mode
			if (_showHistorical) {
				const histVar = _historicalData.get(varConfig.id);
				if (histVar && histVar.data.length > 0) {
					linesData.push({
						id: `hist-${varConfig.id}`,
						color: '#9ca3af',
						points: histVar.data.map((d) => ({ year: d.year, normalized: d.value })),
						rawPoints: histVar.data.map((d) => ({ year: d.year, value: d.value })),
						label: 'Historical',
						format: varConfig.format,
						historical: true
					});
				}
			}
```

**Step 6: Apply dashed styling to historical lines**

In the line rendering section (lines 240-268), modify the `enter` and `update` handlers to apply dashed stroke for historical lines.

Replace the entire lines join block (lines 240-268) with:

```typescript
		lines.join(
			(enter) =>
				enter
					.append('path')
					.attr('class', 'var-line')
					.attr('fill', 'none')
					.attr('stroke-width', (d) => d.historical ? 1.5 : 2)
					.attr('stroke', (d) => d.color)
					.attr('stroke-dasharray', (d) => d.historical ? '6,3' : null)
					.attr('opacity', 0)
					.attr('d', (d) => line(d.points))
					.transition()
					.duration(400)
					.attr('opacity', (d) => d.historical ? 0.6 : 1),
			(update) =>
				update
					.transition()
					.duration(400)
					.attr('stroke', (d) => d.color)
					.attr('stroke-width', (d) => d.historical ? 1.5 : 2)
					.attr('stroke-dasharray', (d) => d.historical ? '6,3' : null)
					.attr('opacity', (d) => d.historical ? 0.6 : 1)
					.attr('d', (d) => line(d.points)),
			(exit) => exit
				.transition()
				.duration(200)
				.attr('opacity', 0)
				.remove()
		);
```

**Step 7: Add historical toggle to legend**

After line 174 (the `legendData` assignment in normal mode), add a "Historical" legend entry:

```typescript
			// Add historical toggle to legend
			if (_historicalData.size > 0) {
				legendData.push({
					id: 'historical',
					label: 'Historical',
					color: '#9ca3af',
					fieldPath: '__historical__',
					visible: _showHistorical
				});
			}
```

Update the legend click handler. In the `enter` handler's `.on('click')` (line 406-408), replace with:

```typescript
					.on('click', (_, d) => {
						if (d.fieldPath === '__historical__') {
							showHistorical.update((v) => !v);
						} else if (!_compareMode) {
							handleLegendToggle(d.fieldPath);
						}
					});
```

And same for the `update` handler's `.on('click')` (line 434-436):

```typescript
				update.on('click', (_, d) => {
					if (d.fieldPath === '__historical__') {
						showHistorical.update((v) => !v);
					} else if (!_compareMode) {
						handleLegendToggle(d.fieldPath);
					}
				});
```

Update the legend icon to show dashed style for historical:

In the legend enter handler, replace the `rect` append (lines 410-415) with:

```typescript
				item.append('rect')
					.attr('width', 12)
					.attr('height', 3)
					.attr('y', 5)
					.attr('rx', 1.5)
					.attr('fill', (d) => d.color)
					.attr('stroke-dasharray', (d) => d.fieldPath === '__historical__' ? '3,2' : null);
```

**Step 8: Add historical values to tooltip**

In the tooltip mousemove handler (lines 515-538), after the existing loop that builds `items`, the historical data is already included because it's part of `linesData`. However, the tooltip label should distinguish historical entries. Modify the item push (line 530-536) to add "(hist.)" when historical:

This is already handled because the `label` field on historical LineDatums includes "hist." suffix (e.g., `Pop. hist.`). No additional change needed.

**Step 9: Verify frontend builds**

```bash
cd frontend && npm run check
```

**Step 10: Commit**

```bash
git add frontend/src/lib/charts/UnifiedChart.svelte
git commit -m "feat(frontend): add historical data overlay to UnifiedChart

Dashed lines at 60% opacity in both normal and compare modes.
Legend toggle to show/hide historical data (default: on)."
```

---

### Task 7: Frontend — Historical overlay on TimeSeriesChart

**Files:**
- Modify: `frontend/src/lib/charts/TimeSeriesChart.svelte`

**Step 1: Add imports**

In `frontend/src/lib/charts/TimeSeriesChart.svelte`, after line 8 (the `chart-config` import), add:

```typescript
	import { showHistorical } from '../stores/chart-ui';
	import { historicalData } from '../stores/historical';
	import { unifiedVariables } from './unified-config';
```

**Step 2: Read stores in $effect**

In the `$effect` block, after line 62 (`const _focusedId = focusedScenarioId;`), add:

```typescript
		const _showHistorical = $showHistorical;
		const _historicalData = $historicalData;
```

**Step 3: Inject historical series**

After the `for (const [id, states] of _data)` loop (after line 77), add:

```typescript
		// Add historical overlay
		if (_showHistorical) {
			const varConfig = unifiedVariables.find((v) => v.fieldPath === _config.fieldPath);
			if (varConfig) {
				const histVar = _historicalData.get(varConfig.id);
				if (histVar && histVar.data.length > 0) {
					allSeries.push({
						id: '__historical__',
						points: histVar.data.map((d) => ({ year: d.year, value: d.value })),
						color: '#9ca3af'
					});
				}
			}
		}
```

**Step 4: Apply dashed styling for historical line**

Replace the lines join block (lines 160-176) with:

```typescript
		lines.join(
			(enter) =>
				enter
					.append('path')
					.attr('class', 'line')
					.attr('fill', 'none')
					.attr('stroke-width', (d) => d.id === '__historical__' ? 1.5 : 2)
					.attr('stroke', (d) => d.color)
					.attr('stroke-dasharray', (d) => d.id === '__historical__' ? '6,3' : null)
					.attr('opacity', (d) => d.id === '__historical__' ? 0.6 : 1)
					.attr('d', (d) => line(d.points)),
			(update) =>
				update
					.transition()
					.duration(300)
					.attr('stroke', (d) => d.color)
					.attr('stroke-width', (d) => d.id === '__historical__' ? 1.5 : 2)
					.attr('stroke-dasharray', (d) => d.id === '__historical__' ? '6,3' : null)
					.attr('opacity', (d) => d.id === '__historical__' ? 0.6 : 1)
					.attr('d', (d) => line(d.points)),
			(exit) => exit.remove()
		);
```

**Step 5: Add historical label to tooltip**

In the tooltip items loop (lines 257-272), modify the item push to label historical data distinctly:

Replace line 270:
```typescript
						items.push({ color: series.color, value: fmt(pt.value), trend });
```
with:
```typescript
						items.push({
							color: series.color,
							value: fmt(pt.value),
							trend: series.id === '__historical__' ? '(hist.)' : trend
						});
```

**Step 6: Verify frontend builds**

```bash
cd frontend && npm run check
```

**Step 7: Commit**

```bash
git add frontend/src/lib/charts/TimeSeriesChart.svelte
git commit -m "feat(frontend): add historical data overlay to TimeSeriesChart

Dashed gray line at 60% opacity. Tooltip shows (hist.) label."
```

---

### Task 8: Documentation — data/historical/README.md

**Files:**
- Create: `data/historical/README.md`

**Step 1: Write the README**

Create `data/historical/README.md` with full documentation of sources, transformations, caveats, and update instructions.

```markdown
# Historical Data

Pre-processed real-world historical data for comparison with World 3 simulation output.
Served by the backend via `GET /api/v1/historical`.

## Variables

### population.csv
- **Source:** World Bank, indicator `SP.POP.TOTL` (World aggregate)
- **URL:** https://data.worldbank.org/indicator/SP.POP.TOTL?locations=1W
- **Units:** persons (matches World 3)
- **Transformation:** none
- **Range:** 1960–2023

### resources.csv
- **Source:** Our World in Data Energy Dataset (Energy Institute / BP Statistical Review)
- **URL:** https://github.com/owid/energy-data
- **Units:** fraction remaining (0–1, 1900=1.0)
- **Transformation:** `fraction_remaining = max(0, 1.0 - cumulative_fossil_production_twh / 10,000,000)`
  - Sum `oil_production + coal_production + gas_production` (TWh/yr) for `country == "World"`
  - Compute cumulative sum from 1900
  - URR estimate: 10,000,000 TWh (≈36,000 EJ total recoverable fossil fuel)
- **Range:** 1900–2023
- **Caveat:** World 3 "resources" is broader than fossil fuels (includes metals/minerals). This is a proxy using the best available aggregate indicator. The URR estimate is debatable — proved reserves have grown over time.

### food.csv
- **Source:** FAOSTAT Food Balance Sheets — Grand Total food supply quantity
- **URL:** https://www.fao.org/faostat/en/#data/FBS
- **Units:** kg/person/year
- **Transformation:** `kcal_per_capita_per_day * 365 / 1200` (1200 kcal/kg weighted average food energy density)
- **Range:** 1961–2022
- **Caveat:** The World 3 1900 starting value (400 kg/capita/yr) is lower than modern FAO values (~900), reflecting genuine improvement in food supply.

### industrial.csv
- **Source:** World Bank `NV.IND.TOTL.KD` (industry value added, constant 2015 USD) ÷ `SP.POP.TOTL`
- **URL:** https://data.worldbank.org/indicator/NV.IND.TOTL.KD
- **Units:** 1975 USD/person/year
- **Transformation:** `(NV.IND.TOTL.KD / SP.POP.TOTL) / 3.51`
  - 3.51 = US GDP deflator ratio (2015 USD → 1975 USD), from World Bank `NY.GDP.DEFL.ZS.AD`
- **Range:** 1960–2023
- **Caveat:** `NV.IND.TOTL.KD` covers industry + construction, the closest match to World 3's "industrial output". The deflator ratio is approximate.

### pollution.csv
- **Source:** NOAA Global Monitoring Laboratory, Mauna Loa CO₂ annual mean
- **URL:** https://gml.noaa.gov/webdata/ccgg/trends/co2/co2_annmean_mlo.txt
- **Units:** index (1970=1.0)
- **Transformation:** `(co2_ppm - 280) / (325.7 - 280)`
  - 280 ppm = pre-industrial CO₂ baseline
  - 325.7 ppm = Mauna Loa CO₂ in 1970
- **Range:** 1959–2025
- **Caveat:** World 3 "persistent pollution" is a composite of all long-lived pollutants, not just CO₂. CO₂ is the best single proxy due to measurement quality and growth trajectory match.

### life-expectancy.csv
- **Source:** World Bank, indicator `SP.DYN.LE00.IN` (life expectancy at birth, World aggregate)
- **URL:** https://data.worldbank.org/indicator/SP.DYN.LE00.IN?locations=1W
- **Units:** years
- **Transformation:** none
- **Range:** 1960–2022

## Updating Data

1. Download fresh data from the source URLs listed above
2. Apply transformations as documented
3. Replace the CSV file, preserving the comment-header format
4. Update the `# retrieved:` date in the header
5. Verify the backend loads correctly: `RUST_LOG=info cargo run --bin world3-api`

## CSV Format

All files follow the same schema:

```csv
# source: <source name>
# url: <download URL>
# units: <World 3 model units>
# transformation: <formula or "none">
# retrieved: <YYYY-MM-DD>
year,value
1960,3.034e9
...
```

## Future

When `world3-ingestion` (Milestone 3) is implemented, these CSVs become the
fallback in the data pipeline: live API → SQLite cache → bundled CSV.
The `GET /api/v1/historical` endpoint contract stays the same.
```

**Step 2: Commit**

```bash
git add data/historical/README.md
git commit -m "docs: add README for historical data sources and transformations"
```

---

### Task 9: Integration testing and verification

**Files:** None (testing only)

**Step 1: Run all backend tests**

```bash
cargo test --workspace
```

Expected: All tests pass including the new `historical::tests` module.

**Step 2: Run cargo clippy**

```bash
cargo clippy --workspace -- -D warnings
```

Expected: No warnings.

**Step 3: Run all frontend tests**

```bash
cd frontend && npm run check && npm test
```

Expected: All tests pass, TypeScript check passes.

**Step 4: Run frontend build**

```bash
cd frontend && npm run build
```

Expected: Build succeeds (verifies no type errors in Svelte components).

**Step 5: Manual smoke test (if server available)**

```bash
# Terminal 1: Start backend
STATIC_DIR=frontend/build RUST_LOG=info cargo run --bin world3-api

# Terminal 2: Test historical endpoint
curl http://localhost:8080/api/v1/historical | jq '.[] | .variable'
curl http://localhost:8080/api/v1/historical/population | jq '.data | length'
```

Expected: Returns all 6 variable names; population has ~64 data points.

**Step 6: Commit (if any fixes needed)**

```bash
git add -A && git commit -m "fix: address integration test findings"
```

---

## Task Dependency Graph

```
Task 1 (CSVs) ──────┐
                     ├──→ Task 3 (API endpoint) ──→ Task 5 (frontend loading) ──→ Task 6 (UnifiedChart) ──→ Task 9 (verify)
Task 2 (parser) ─────┘                                                        ──→ Task 7 (TimeSeriesChart) ↗
                                                   Task 4 (store/API) ─────────┘
                                                   Task 8 (docs) ─────────────────────────────────────────────↗
```

Tasks 1+2 can be done in parallel. Tasks 4 and 8 can be done in parallel with each other. Tasks 6 and 7 can be done in parallel.
