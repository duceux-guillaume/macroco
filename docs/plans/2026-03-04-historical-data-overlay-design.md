# Historical Data Overlay Design

**Date:** 2026-03-04
**Status:** Approved
**Scope:** Add real-world historical data overlays to all 6 D3 chart variables

## Goal

Show real-world historical data alongside World 3 simulation curves on the frontend charts. Users can visually compare what actually happened versus what the model predicts, making the simulation's accuracy (and divergences) immediately apparent.

## Architecture

```
data/historical/*.csv  →  Backend API endpoint  →  Frontend store  →  D3 chart overlay
```

Pre-bundled static CSVs served by a backend endpoint. This path naturally evolves to database-backed serving when the ingestion crate (Milestone 3) is implemented — same API contract, different data source.

## Data Sources

| Variable | Source | Indicator/File | Range | Transform |
|---|---|---|---|---|
| Population | World Bank | `SP.POP.TOTL` | 1960–2023 | None (units match) |
| Resources Remaining | OWID Energy Data | `owid-energy-data.csv` | 1900–2023 | Cumulate production → fraction of URR |
| Food Per Capita | FAOSTAT | Food Balance Sheets | 1961–2022 | Grand Total food supply (kg/capita/yr) |
| Industrial Output/Capita | World Bank | `NV.IND.TOTL.KD` + `SP.POP.TOTL` | 1960–2023 | Per capita, then deflate 2015→1975 USD (÷3.51) |
| Pollution Index | NOAA GML | `co2_annmean_mlo.txt` | 1959–2025 | `(CO2 - 280) / (325.7 - 280)`, normalized 1970=1.0 |
| Life Expectancy | World Bank | `SP.DYN.LE00.IN` | 1960–2022 | None (units match) |

All sources are free and require no API keys.

## CSV Format

One file per variable in `data/historical/`. Uniform schema with provenance headers:

```csv
# source: World Bank SP.POP.TOTL
# url: https://data.worldbank.org/indicator/SP.POP.TOTL?locations=1W
# units: persons (World 3 model units)
# transformation: none
# retrieved: 2026-03-04
year,value
1960,3.034e9
1961,3.075e9
...
```

Files:
- `population.csv`
- `resources_remaining.csv`
- `food_per_capita.csv`
- `industrial_output_per_capita.csv`
- `pollution_index.csv`
- `life_expectancy.csv`

## Transformations

### Resources Remaining (most complex)

Raw data: OWID Energy Data — `oil_production`, `coal_production`, `gas_production` (TWh/yr), filtered for `country == "World"`.

```
cumulative_extraction(year) = SUM(oil + coal + gas production) from 1900 to year
URR_ESTIMATE = ~10,000,000 TWh  (≈36,000 EJ total recoverable fossil fuel)
fraction_remaining(year) = max(0, 1.0 - cumulative_extraction(year) / URR_ESTIMATE)
```

**Caveat:** World 3 "resources" includes metals/minerals, not just fossil fuels. This is a proxy. The URR estimate is debatable — proved reserves have grown over time.

### Industrial Output Per Capita

Raw data: World Bank `NV.IND.TOTL.KD` (constant 2015 USD) and `SP.POP.TOTL`.

```
per_capita_2015usd = NV.IND.TOTL.KD / SP.POP.TOTL
DEFLATOR_RATIO = 3.51  (2015 USD / 1975 USD, from US GDP deflator)
per_capita_1975usd = per_capita_2015usd / DEFLATOR_RATIO
```

**Caveat:** Deflator ratio should be verified from World Bank `NY.GDP.DEFL.ZS.AD`. The `NV.IND.TOTL.KD` covers industry+construction, which is the closest match to World 3's "industrial output".

### Pollution Index

Raw data: NOAA Mauna Loa CO2 annual mean (ppm).

```
CO2_PREINDUSTRIAL = 280.0  (ppm)
CO2_1970 = 325.7  (ppm, from Mauna Loa record)
pollution_index(year) = (CO2(year) - CO2_PREINDUSTRIAL) / (CO2_1970 - CO2_PREINDUSTRIAL)
```

**Caveat:** World 3 "persistent pollution" is a composite of all long-lived pollutants. CO2 is the best single proxy due to measurement quality and growth trajectory match.

### Food Per Capita

Raw data: FAOSTAT Food Balance Sheets — "Grand Total" food supply quantity (kg/capita/yr).

No arithmetic transformation needed — units are close to World 3's `food_per_capita` (kg/person/year vegetable-equivalent). The World 3 initial value of 400 kg/capita/yr (1900) is lower than modern FAO values (~900), reflecting real improvement.

## Backend API

### New Endpoint

`GET /api/v1/historical/{variable_id}`

Response:
```json
{
  "variable": "population",
  "source": "World Bank SP.POP.TOTL",
  "units": "persons",
  "transformation": "none",
  "data": [
    {"year": 1960, "value": 3.034e9},
    {"year": 1961, "value": 3.075e9}
  ]
}
```

`GET /api/v1/historical` — returns all 6 variables in a single response (for batch loading).

### Implementation

- CSV files loaded once at server startup into `AppState`
- Parsed into `HashMap<String, HistoricalVariable>` keyed by variable ID
- Variable IDs match frontend `unifiedVariables` config IDs: `population`, `resources`, `food`, `industrial`, `pollution`, `life-expectancy`

## Frontend

### Store

New file: `frontend/src/lib/stores/historicalStore.ts`

```typescript
type HistoricalSeries = { year: number; value: number }[];
type HistoricalVariable = {
  data: HistoricalSeries;
  source: string;
  units: string;
  transformation: string;
};
type HistoricalData = Map<string, HistoricalVariable>;

export const historicalData: Writable<HistoricalData> = writable(new Map());
export const showHistorical: Writable<boolean> = writable(true);
```

### Data Loading

In `+layout.svelte`, after fetching scenarios, fetch `GET /api/v1/historical` and populate `historicalData` store.

### Chart Rendering

Both `UnifiedChart.svelte` and `TimeSeriesChart.svelte`:

1. Read `historicalData` and `showHistorical` from store
2. For each charted variable, look up matching historical series by variable ID
3. Render as **dashed line** in **muted/lighter shade** of the variable's color (same hue, ~50% opacity)
4. Historical line shares the same x/y scales as simulation curves
5. Line ends where data ends (~2023); simulation continues to 2100

**UnifiedChart normalized mode:** Historical data normalized using the same min/max as simulation data, keeping the overlay visually aligned.

**Toggle:** Button in chart legend area — "Historical" on/off. Bound to `showHistorical` store. Default: on.

**Legend:** Historical series shown with dashed line icon and label like "Population (historical)".

## Documentation

`data/historical/README.md` — Single reference documenting:
- Each variable's source, URL, raw format, units
- Transformation formula with rationale
- Caveats and limitations
- Date of last data retrieval
- Instructions for updating data

## Out of Scope

- Live data fetching (Milestone 3)
- Pre-1960 reconstructions (Maddison, Gapminder, ice cores) — can be added later
- SparklineChart overlay (add if time permits, but not required)
- Error bars or uncertainty ranges on historical data

## Future Evolution

When `world3-ingestion` (Milestone 3) is implemented:
1. Backend swaps CSV file reads for database queries
2. Same `GET /api/v1/historical/{variable}` endpoint, same response format
3. Frontend unchanged
4. CSVs become the fallback in the ingestion crate's chain: live API → SQLite → bundled CSV
