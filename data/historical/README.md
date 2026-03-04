# Historical Data

Pre-processed real-world historical data for comparison with World 3 simulation output.
Served by the backend via `GET /api/v1/historical`.

## Variables

### population.csv
- **Source:** World Bank, indicator `SP.POP.TOTL` (World aggregate)
- **URL:** https://data.worldbank.org/indicator/SP.POP.TOTL?locations=1W
- **Units:** persons (matches World 3)
- **Transformation:** none
- **Range:** 1960-2023

### resources_remaining.csv
- **Source:** Our World in Data Energy Dataset (Energy Institute / BP Statistical Review)
- **URL:** https://github.com/owid/energy-data
- **Units:** fraction remaining (0-1, 1900=1.0)
- **Transformation:** `fraction_remaining = max(0, 1.0 - cumulative_fossil_production_twh / 10,000,000)`
  - Sum `oil_production + coal_production + gas_production` (TWh/yr) for `country == "World"`
  - Compute cumulative sum from 1900
  - URR estimate: 10,000,000 TWh (~36,000 EJ total recoverable fossil fuel)
- **Range:** 1900-2023
- **Caveat:** World 3 "resources" is broader than fossil fuels (includes metals/minerals). This is a proxy using the best available aggregate indicator. The URR estimate is debatable.

### food_per_capita.csv
- **Source:** FAOSTAT Food Balance Sheets — Global average dietary energy supply
- **URL:** https://www.fao.org/faostat/en/#data/FBS
- **Units:** kg/person/year (vegetable-equivalent)
- **Transformation:** `kcal_per_capita_per_day * 365 / 1200` (1200 kcal/kg weighted average food energy density)
- **Range:** 1961-2022
- **Caveat:** The World 3 1900 starting value (400 kg/capita/yr) is lower than modern values (~900), reflecting genuine improvement in food supply. The kcal-to-kg conversion uses a global diet average energy density.

### industrial_output_per_capita.csv
- **Source:** World Bank `NV.IND.TOTL.KD` (industry value added, constant 2015 USD) / `SP.POP.TOTL`
- **URL:** https://data.worldbank.org/indicator/NV.IND.TOTL.KD
- **Units:** 1975 USD/person/year
- **Transformation:** `(NV.IND.TOTL.KD / SP.POP.TOTL) / 3.51`
  - 3.51 = US GDP deflator ratio (2015 USD -> 1975 USD), from World Bank `NY.GDP.DEFL.ZS.AD`
  - 1960-1993: estimated from `NY.GDP.MKTP.KD * industrial_share`, splice-adjusted to match 1994 actual data
  - 1994-2023: directly from `NV.IND.TOTL.KD`
- **Range:** 1960-2023
- **Caveat:** `NV.IND.TOTL.KD` covers industry + construction, the closest match to World 3's "industrial output". The deflator ratio is approximate. Pre-1994 values are estimated from GDP.

### pollution_index.csv
- **Source:** NOAA Global Monitoring Laboratory, Mauna Loa CO2 annual mean
- **URL:** https://gml.noaa.gov/webdata/ccgg/trends/co2/co2_annmean_mlo.txt
- **Units:** index (1970=1.0)
- **Transformation:** `(co2_ppm - 280) / (325.68 - 280)`
  - 280 ppm = pre-industrial CO2 baseline
  - 325.68 ppm = Mauna Loa CO2 in 1970
- **Range:** 1959-2025
- **Caveat:** World 3 "persistent pollution" is a composite of all long-lived pollutants, not just CO2. CO2 is the best single proxy due to measurement quality and growth trajectory match.

### life_expectancy.csv
- **Source:** World Bank, indicator `SP.DYN.LE00.IN` (life expectancy at birth, World aggregate)
- **URL:** https://data.worldbank.org/indicator/SP.DYN.LE00.IN?locations=1W
- **Units:** years
- **Transformation:** none
- **Range:** 1960-2022

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
1960,3021512598
...
```

## Future

When `world3-ingestion` (Milestone 3) is implemented, these CSVs become the
fallback in the data pipeline: live API -> SQLite cache -> bundled CSV.
The `GET /api/v1/historical` endpoint contract stays the same.
