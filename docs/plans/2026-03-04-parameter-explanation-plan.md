# Parameter Explanation Panel — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a rich, educational parameter explanation panel that shows beginner/expert descriptions, impact previews with sparkline charts, feedback loops, and related variables when users click an info icon on any simulation slider.

**Architecture:** New `ParameterInfoPanel` component (modeled after `VariableInfoPanel`), extended `ParameterInfo` data model, new stores for parameter selection and chart highlighting, and chart line dimming via the `highlightedVariables` derived store. Sparkline reads from existing `simulationResults` store (no new API calls).

**Tech Stack:** SvelteKit 5 (runes), TypeScript, D3 v7, Rust (Axum backend for 3 new parameter descriptors)

---

### Task 1: Backend — Add 3 missing parameters to `parameter_descriptors()`

**Files:**
- Modify: `crates/world3-core/src/model/params.rs:258` (insert before closing `]`)

**Step 1: Add the 3 parameter descriptors**

Insert before the closing `]` on line 259 of `params.rs`, after the `pollution_control` entry:

```rust
        ParameterDescriptor {
            field: "service_depreciation_rate".into(),
            label: "Service Capital Depreciation".into(),
            unit: "yr⁻¹".into(),
            min: 0.02, max: 0.10, default: 0.05, step: 0.005,
            sector: "capital".into(),
            description: "Annual fraction of service capital that wears out.".into(),
        },
        ParameterDescriptor {
            field: "subsistence_food_per_capita".into(),
            label: "Subsistence Food Level".into(),
            unit: "kg/person/yr".into(),
            min: 150.0, max: 400.0, default: 230.0, step: 10.0,
            sector: "agriculture".into(),
            description: "Minimum food per person needed for basic health.".into(),
        },
        ParameterDescriptor {
            field: "initial_nnr_fraction".into(),
            label: "Initial Resource Level".into(),
            unit: "fraction".into(),
            min: 0.1, max: 1.0, default: 1.0, step: 0.05,
            sector: "resources".into(),
            description: "Starting level of non-renewable resources as a fraction of full endowment.".into(),
        },
```

**Step 2: Run tests to verify**

Run: `cargo test --workspace && cargo clippy --workspace -- -D warnings`
Expected: All tests pass, no clippy warnings.

**Step 3: Commit**

```bash
git add crates/world3-core/src/model/params.rs
git commit -m "feat: expose 3 hidden parameters in slider schema"
```

---

### Task 2: Extend `ParameterInfo` interface

**Files:**
- Modify: `frontend/src/lib/content/variable-descriptions.ts:14-20`
- Modify: `frontend/src/lib/content/variable-descriptions.test.ts:21-27`

**Step 1: Write the failing test**

In `variable-descriptions.test.ts`, update `REQUIRED_PARAM_FIELDS` (line 21-27) to include new fields:

```typescript
const REQUIRED_PARAM_FIELDS: (keyof ParameterInfo)[] = [
	'name',
	'unit',
	'sector',
	'beginner',
	'expert',
	'feedbackLoops',
	'relatedVariables',
	'impact'
];
```

Add a new test after the "has at least 10 parameters" test (after line 101):

```typescript
	it('all feedbackLoop IDs reference existing feedbackLoops', () => {
		const validLoopIds = new Set(Object.keys(feedbackLoops));
		for (const [key, info] of Object.entries(parameterDescriptions)) {
			for (const loopId of info.feedbackLoops) {
				expect(
					validLoopIds.has(loopId),
					`${key} references unknown feedback loop: ${loopId}`
				).toBe(true);
			}
		}
	});

	it('all relatedVariables reference existing variableDescriptions', () => {
		const validKeys = new Set(Object.keys(variableDescriptions));
		for (const [key, info] of Object.entries(parameterDescriptions)) {
			for (const ref of info.relatedVariables) {
				expect(validKeys.has(ref), `${key} references unknown variable: ${ref}`).toBe(true);
			}
		}
	});

	it('all impact.sparklineVariable reference existing variableDescriptions', () => {
		const validKeys = new Set(Object.keys(variableDescriptions));
		for (const [key, info] of Object.entries(parameterDescriptions)) {
			expect(
				validKeys.has(info.impact.sparklineVariable),
				`${key} impact.sparklineVariable references unknown variable: ${info.impact.sparklineVariable}`
			).toBe(true);
		}
	});
```

**Step 2: Run test to verify it fails**

Run: `cd frontend && npm test -- --run variable-descriptions`
Expected: FAIL — `ParameterInfo` type does not have `feedbackLoops`, `relatedVariables`, `impact`.

**Step 3: Update the `ParameterInfo` interface**

In `variable-descriptions.ts`, replace lines 14-20:

```typescript
export interface ParameterImpact {
	increase: string;
	decrease: string;
	sparklineVariable: string;
}

export interface ParameterInfo {
	name: string;
	unit: string;
	sector: string;
	beginner: string;
	expert: string;
	feedbackLoops: string[];
	relatedVariables: string[];
	impact: ParameterImpact;
}
```

**Step 4: Run test to verify it still fails (data not yet populated)**

Run: `cd frontend && npm test -- --run variable-descriptions`
Expected: FAIL — existing entries don't have the new fields yet. This confirms the test is checking the right thing.

**Step 5: Commit interface change**

```bash
git add frontend/src/lib/content/variable-descriptions.ts frontend/src/lib/content/variable-descriptions.test.ts
git commit -m "feat: extend ParameterInfo interface with feedbackLoops, relatedVariables, impact"
```

---

### Task 3: Populate parameter content with new fields

**Files:**
- Modify: `frontend/src/lib/content/variable-descriptions.ts:269-382` (all 13 `parameterDescriptions` entries)

**Step 1: Add `feedbackLoops`, `relatedVariables`, and `impact` to all 13 entries**

Each existing entry gets 3 new fields. Here is the complete updated `parameterDescriptions` object. Replace lines 268-383:

```typescript
export const parameterDescriptions: Record<string, ParameterInfo> = {
	family_planning_year: {
		name: 'Family Planning Start Year',
		unit: 'year',
		sector: 'Population',
		beginner:
			'The year when family planning programs become fully effective. Earlier = earlier fertility decline.',
		expert:
			'Controls the ramp function: fp_ramp = clamp((time − 1900) / (fp_year − 1900), 0, 1). Multiplied by efficacy to get effective family planning input.',
		feedbackLoops: ['demographic-transition'],
		relatedVariables: ['population.population', 'population.fertility_rate', 'population.birth_rate'],
		impact: {
			increase: 'Delays fertility decline — population grows larger before stabilizing',
			decrease: 'Earlier fertility decline — population peaks sooner and at a lower level',
			sparklineVariable: 'population.population'
		}
	},
	family_planning_efficacy: {
		name: 'Family Planning Efficacy',
		unit: '0–1',
		sector: 'Population',
		beginner:
			'How effective family planning programs are at reducing birth rates. 0 = no effect, 1 = maximum effect.',
		expert:
			'Scales the family_planning_multiplier lookup input. At efficacy=1.0 and full ramp, fertility multiplier ≈ 0.40.',
		feedbackLoops: ['demographic-transition'],
		relatedVariables: ['population.population', 'population.fertility_rate', 'population.birth_rate'],
		impact: {
			increase: 'Stronger fertility reduction — smaller peak population, less resource pressure',
			decrease: 'Weaker family planning — higher birth rates persist longer',
			sparklineVariable: 'population.population'
		}
	},
	health_investment_multiplier: {
		name: 'Health Investment Multiplier',
		unit: 'multiplier',
		sector: 'Population',
		beginner:
			'How much the economy invests in healthcare. Higher values mean better health services and longer life expectancy.',
		expert: 'Scales service_output_per_capita input to life_exp_multiplier_health lookup.',
		feedbackLoops: ['demographic-transition', 'food-population'],
		relatedVariables: ['population.life_expectancy', 'population.death_rate', 'population.population'],
		impact: {
			increase: 'Better health → longer life expectancy → slower population decline',
			decrease: 'Worse health → higher death rates → faster population decline',
			sparklineVariable: 'population.life_expectancy'
		}
	},
	industrial_depreciation_rate: {
		name: 'Industrial Depreciation Rate',
		unit: 'fraction/yr',
		sector: 'Capital',
		beginner:
			'How fast factories and machines wear out. Higher = capital decays faster, requiring more investment just to maintain.',
		expert:
			'Used in d(IC)/dt = investment − IC × depreciation_rate. Default 0.05 = 20-year average capital lifetime.',
		feedbackLoops: ['resource-collapse'],
		relatedVariables: ['capital.industrial_capital', 'capital.industrial_output', 'capital.industrial_output_per_capita'],
		impact: {
			increase: 'Capital wears out faster — economy needs more investment just to stay level',
			decrease: 'Capital lasts longer — more output available for services and consumption',
			sparklineVariable: 'capital.industrial_output_per_capita'
		}
	},
	service_depreciation_rate: {
		name: 'Service Depreciation Rate',
		unit: 'fraction/yr',
		sector: 'Capital',
		beginner:
			'How fast service infrastructure (hospitals, schools) wears out.',
		expert:
			'Used in d(SC)/dt = service_investment − SC × depreciation_rate. Default 0.05.',
		feedbackLoops: ['demographic-transition'],
		relatedVariables: ['capital.service_output_per_capita', 'population.life_expectancy'],
		impact: {
			increase: 'Services decay faster — health and education quality drops',
			decrease: 'Services last longer — sustained life expectancy improvements',
			sparklineVariable: 'capital.service_output_per_capita'
		}
	},
	technology_growth_rate: {
		name: 'Technology Growth Rate',
		unit: 'fraction/yr',
		sector: 'Capital',
		beginner:
			'Annual improvement in how efficiently capital produces output. Compounds over time — even small rates have big long-term effects.',
		expert:
			'tech_multiplier = (1 + rate)^max(time−1970, 0). Applied to productive capital before ICOR division.',
		feedbackLoops: ['resource-collapse'],
		relatedVariables: ['capital.industrial_output', 'capital.industrial_output_per_capita', 'resources.fraction_remaining'],
		impact: {
			increase: 'More output per unit capital — delays resource-driven collapse',
			decrease: 'Slower technological progress — economy hits limits earlier',
			sparklineVariable: 'capital.industrial_output_per_capita'
		}
	},
	investment_rate: {
		name: 'Investment Rate',
		unit: 'fraction',
		sector: 'Capital',
		beginner:
			'What fraction of industrial output is reinvested in building new capital. Higher = faster growth but less available for services and agriculture.',
		expert:
			'Fraction of industrial_output allocated to gross investment. d(IC)/dt investment term = IO × investment_rate.',
		feedbackLoops: ['resource-collapse'],
		relatedVariables: ['capital.industrial_capital', 'capital.industrial_output', 'capital.service_output_per_capita'],
		impact: {
			increase: 'Faster capital growth but less output for services — trade-off between growth and welfare',
			decrease: 'Slower capital growth but more services available — better short-term welfare',
			sparklineVariable: 'capital.industrial_capital'
		}
	},
	agricultural_technology: {
		name: 'Agricultural Technology',
		unit: 'multiplier',
		sector: 'Agriculture',
		beginner:
			'Multiplier on crop yields from improved farming techniques — better seeds, irrigation, precision agriculture.',
		expert: 'Direct multiplier on land_yield: LY = 600 × LYMC × LYMAP × agri_tech.',
		feedbackLoops: ['food-population', 'pollution-food'],
		relatedVariables: ['agriculture.food_per_capita', 'agriculture.land_yield', 'agriculture.food'],
		impact: {
			increase: 'More food per hectare — delays food crisis, supports larger population',
			decrease: 'Lower yields — food shortages arrive earlier',
			sparklineVariable: 'agriculture.food_per_capita'
		}
	},
	land_protection_fraction: {
		name: 'Land Protection',
		unit: 'fraction (0–0.5)',
		sector: 'Agriculture',
		beginner:
			'How much farmland is protected from erosion through conservation practices. 0 = no protection, 0.5 = half of erosion prevented.',
		expert:
			'Reduces erosion: erosion × (1 − land_protection_fraction). Clamped to [0, 0.5].',
		feedbackLoops: ['food-population'],
		relatedVariables: ['agriculture.arable_land', 'agriculture.food_per_capita', 'agriculture.land_yield'],
		impact: {
			increase: 'Less farmland lost to erosion — sustained food production capacity',
			decrease: 'More erosion — arable land shrinks faster, food production drops',
			sparklineVariable: 'agriculture.arable_land'
		}
	},
	subsistence_food_per_capita: {
		name: 'Subsistence Food Level',
		unit: 'kg/person/yr',
		sector: 'Agriculture',
		beginner:
			'The minimum food per person needed for basic health. Below this level, life expectancy drops sharply.',
		expert:
			'Denominator in food_ratio = FPC / subsistence_food. Drives multiple lookup tables. Default 230 kg/yr.',
		feedbackLoops: ['food-population'],
		relatedVariables: ['agriculture.food_per_capita', 'population.life_expectancy', 'population.death_rate'],
		impact: {
			increase: 'Higher bar for adequate nutrition — more people classified as food-insecure',
			decrease: 'Lower nutrition threshold — fewer people in food crisis at same production',
			sparklineVariable: 'agriculture.food_per_capita'
		}
	},
	resource_efficiency: {
		name: 'Resource Efficiency',
		unit: 'multiplier',
		sector: 'Resources',
		beginner:
			'How efficiently resources are used. Higher values mean the economy gets more output per unit of resource consumed. Technology preset uses 4x.',
		expert:
			'Divides extraction rate: extraction = pop × IOPC × coeff / resource_efficiency.',
		feedbackLoops: ['resource-collapse', 'population-resource'],
		relatedVariables: ['resources.nonrenewable_resources', 'resources.fraction_remaining', 'capital.industrial_output'],
		impact: {
			increase: 'Resources last longer — industrial output sustained further into the future',
			decrease: 'Faster resource depletion — earlier industrial collapse',
			sparklineVariable: 'resources.fraction_remaining'
		}
	},
	initial_nnr_fraction: {
		name: 'Initial Resource Level',
		unit: 'fraction (0–1)',
		sector: 'Resources',
		beginner:
			'Starting level of non-renewable resources. 1.0 = full initial endowment. Lower values simulate a world where resources are already partially depleted.',
		expert: 'Initial condition for nonrenewable_resources ODE stock.',
		feedbackLoops: ['resource-collapse'],
		relatedVariables: ['resources.nonrenewable_resources', 'resources.fraction_remaining', 'capital.industrial_output_per_capita'],
		impact: {
			increase: 'More starting resources — delays the resource depletion crisis',
			decrease: 'Fewer starting resources — collapse arrives much sooner',
			sparklineVariable: 'resources.nonrenewable_resources'
		}
	},
	pollution_control: {
		name: 'Pollution Control',
		unit: 'fraction (0–1)',
		sector: 'Pollution',
		beginner:
			'How much pollution is prevented at the source. 0 = no control, 0.8 = 80% of pollution eliminated before it enters the environment.',
		expert:
			'generation = (gen_industry + gen_agriculture) × (1 − pollution_control). Clamped to [0, 1].',
		feedbackLoops: ['pollution-food', 'pollution-tipping'],
		relatedVariables: ['pollution.persistent_pollution', 'pollution.pollution_index', 'agriculture.food_per_capita'],
		impact: {
			increase: 'Less pollution — protects food production and avoids pollution tipping point',
			decrease: 'More pollution accumulates — food yields drop, pollution may spiral',
			sparklineVariable: 'pollution.pollution_index'
		}
	}
};
```

**Step 2: Run tests to verify they pass**

Run: `cd frontend && npm test -- --run variable-descriptions`
Expected: PASS — all entries have the new fields, all references are valid.

**Step 3: Commit**

```bash
git add frontend/src/lib/content/variable-descriptions.ts
git commit -m "feat: add feedbackLoops, relatedVariables, impact to all parameter descriptions"
```

---

### Task 4: Add `selectedParameterId` and `highlightedVariables` stores

**Files:**
- Modify: `frontend/src/lib/stores/info.ts`
- Modify: `frontend/src/lib/stores/stores.test.ts`

**Step 1: Write the failing tests**

Add to the end of `stores.test.ts`:

```typescript
import { selectedVariableId, selectedParameterId, highlightedVariables } from './info';

// Add to beforeEach:
// selectedVariableId.set(null);
// selectedParameterId.set(null);

describe('selectedParameterId', () => {
	it('defaults to null', () => {
		expect(get(selectedParameterId)).toBeNull();
	});

	it('clears selectedVariableId when set', () => {
		selectedVariableId.set('population.population');
		selectedParameterId.set('resource_efficiency');
		expect(get(selectedVariableId)).toBeNull();
		expect(get(selectedParameterId)).toBe('resource_efficiency');
	});

	it('is cleared when selectedVariableId is set', () => {
		selectedParameterId.set('resource_efficiency');
		selectedVariableId.set('population.population');
		expect(get(selectedParameterId)).toBeNull();
		expect(get(selectedVariableId)).toBe('population.population');
	});
});

describe('highlightedVariables', () => {
	it('returns empty set when no parameter selected', () => {
		selectedParameterId.set(null);
		expect(get(highlightedVariables).size).toBe(0);
	});

	it('returns related variables for selected parameter', () => {
		selectedParameterId.set('resource_efficiency');
		const highlighted = get(highlightedVariables);
		expect(highlighted.has('resources.nonrenewable_resources')).toBe(true);
		expect(highlighted.has('resources.fraction_remaining')).toBe(true);
		expect(highlighted.has('capital.industrial_output')).toBe(true);
	});

	it('returns empty set for unknown parameter', () => {
		selectedParameterId.set('nonexistent_param');
		expect(get(highlightedVariables).size).toBe(0);
	});
});
```

**Step 2: Run tests to verify they fail**

Run: `cd frontend && npm test -- --run stores`
Expected: FAIL — `selectedParameterId` and `highlightedVariables` don't exist yet.

**Step 3: Implement the stores**

Replace `frontend/src/lib/stores/info.ts` with:

```typescript
import { writable, derived } from 'svelte/store';
import { parameterDescriptions } from '$lib/content/variable-descriptions';

/** The field path of the currently selected variable for the info panel (null = closed). */
export const selectedVariableId = writable<string | null>(null);

/** The field name of the currently selected parameter for the info panel (null = closed). */
export const selectedParameterId = writable<string | null>(null);

// Mutual exclusion: setting one clears the other
selectedVariableId.subscribe((v) => {
	if (v !== null) selectedParameterId.set(null);
});
selectedParameterId.subscribe((p) => {
	if (p !== null) selectedVariableId.set(null);
});

/** Set of variable field paths to highlight on the chart when a parameter is selected. */
export const highlightedVariables = derived(selectedParameterId, ($paramId) => {
	if (!$paramId) return new Set<string>();
	const info = parameterDescriptions[$paramId];
	return new Set(info?.relatedVariables ?? []);
});
```

**Step 4: Update `stores.test.ts` beforeEach**

Add to the existing `beforeEach` block (line 15-21):

```typescript
import { selectedVariableId, selectedParameterId, highlightedVariables } from './info';

// Inside existing beforeEach:
selectedVariableId.set(null);
selectedParameterId.set(null);
```

**Step 5: Run tests to verify they pass**

Run: `cd frontend && npm test -- --run stores`
Expected: PASS.

**Step 6: Commit**

```bash
git add frontend/src/lib/stores/info.ts frontend/src/lib/stores/stores.test.ts
git commit -m "feat: add selectedParameterId and highlightedVariables stores"
```

---

### Task 5: Add info icon to `ParameterSlider`

**Files:**
- Modify: `frontend/src/components/ParameterSlider.svelte`
- Modify: `frontend/src/components/ParameterSliders.svelte`

**Step 1: Add `oninfo` callback to `ParameterSlider`**

Update the Props interface in `ParameterSlider.svelte` (lines 4-8):

```typescript
	interface Props {
		descriptor: ParameterDescriptor;
		value: number;
		onchange: (field: string, value: number) => void;
		oninfo: (field: string) => void;
	}

	let { descriptor, value, onchange, oninfo }: Props = $props();
```

**Step 2: Add info button to the markup**

Replace the label section (lines 23-26) with:

```svelte
	<div class="slider-header">
		<div class="label-group">
			<button class="info-btn" onclick={() => oninfo(descriptor.field)} title="Learn about this parameter" aria-label={`Info about ${descriptor.label}`}>
				&#x24D8;
			</button>
			<label for={descriptor.field}>
				{descriptor.label}
			</label>
		</div>
		<span class="slider-value">
			{value.toFixed(descriptor.step < 0.01 ? 3 : descriptor.step < 0.1 ? 2 : 1)}
			<span class="unit">{descriptor.unit}</span>
		</span>
	</div>
```

**Step 3: Add styles for info button**

Add to the `<style>` block:

```css
	.label-group {
		display: flex;
		align-items: center;
		gap: 4px;
	}
	.info-btn {
		background: none;
		border: none;
		cursor: pointer;
		font-size: 14px;
		color: var(--text-secondary);
		padding: 0;
		line-height: 1;
		opacity: 0.5;
		transition: opacity 0.15s, color 0.15s;
	}
	.info-btn:hover {
		opacity: 1;
		color: var(--accent);
	}
```

Remove `cursor: help;` from the existing `label` style (line 66).

**Step 4: Wire `oninfo` in `ParameterSliders`**

In `ParameterSliders.svelte`, add import and handler:

```typescript
import { selectedParameterId } from '$lib/stores/info';

function handleParamInfo(field: string) {
	selectedParameterId.set(field);
}
```

Update the `ParameterSlider` usage (lines 74-78):

```svelte
							<ParameterSlider
								descriptor={desc}
								value={getParamValue(params, desc.field)}
								onchange={handleParamChange}
								oninfo={handleParamInfo}
							/>
```

**Step 5: Run frontend check**

Run: `cd frontend && npm run check`
Expected: No type errors.

**Step 6: Commit**

```bash
git add frontend/src/components/ParameterSlider.svelte frontend/src/components/ParameterSliders.svelte
git commit -m "feat: add info icon button to parameter sliders"
```

---

### Task 6: Create `ParameterInfoPanel` component

**Files:**
- Create: `frontend/src/components/ParameterInfoPanel.svelte`

**Step 1: Create the panel component**

Create `frontend/src/components/ParameterInfoPanel.svelte`:

```svelte
<script lang="ts">
	import { selectedParameterId, selectedVariableId } from '$lib/stores/info';
	import {
		parameterDescriptions,
		feedbackLoops,
		variableDescriptions,
		type ParameterInfo,
		type FeedbackLoopInfo
	} from '$lib/content/variable-descriptions';

	let showExpert = $state(false);
	let parameterId = $state<string | null>(null);

	const unsub = selectedParameterId.subscribe((p) => (parameterId = p));

	let info = $derived<ParameterInfo | null>(
		parameterId ? parameterDescriptions[parameterId] ?? null : null
	);

	let relatedLoops = $derived<FeedbackLoopInfo[]>(
		info
			? info.feedbackLoops
					.map((id) => feedbackLoops[id])
					.filter((l): l is FeedbackLoopInfo => l != null)
			: []
	);

	let relatedVars = $derived(
		info
			? info.relatedVariables
					.map((path) => {
						const desc = variableDescriptions[path];
						return desc ? { path, name: desc.name } : null;
					})
					.filter((v): v is { path: string; name: string } => v != null)
			: []
	);

	function close() {
		selectedParameterId.set(null);
	}

	function selectVariable(path: string) {
		selectedVariableId.set(path);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') close();
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if info && parameterId}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="panel-backdrop" onclick={close}></div>
	<div class="info-panel" role="dialog" aria-label="Parameter information">
		<div class="panel-header">
			<div>
				<h2>{info.name}</h2>
				<span class="meta">{info.sector} · {info.unit}</span>
			</div>
			<button class="close-btn" onclick={close} aria-label="Close panel">&times;</button>
		</div>

		<div class="panel-body">
			<section>
				<p class="description">{info.beginner}</p>
			</section>

			<section>
				<button class="toggle-btn" onclick={() => (showExpert = !showExpert)}>
					{showExpert ? '▾' : '▸'} Technical Detail
				</button>
				{#if showExpert}
					<p class="expert">{info.expert}</p>
				{/if}
			</section>

			<section>
				<h3>Impact</h3>
				<div class="impact-card increase">
					<span class="impact-arrow">&#x2191;</span>
					<p>{info.impact.increase}</p>
				</div>
				<div class="impact-card decrease">
					<span class="impact-arrow">&#x2193;</span>
					<p>{info.impact.decrease}</p>
				</div>
			</section>

			{#if relatedLoops.length > 0}
				<section>
					<h3>Feedback Loops</h3>
					{#each relatedLoops as loop}
						<div class="loop-card">
							<div class="loop-header">
								<span class="loop-type" class:reinforcing={loop.type === 'reinforcing'} class:stabilizing={loop.type === 'stabilizing'}>
									{loop.type === 'reinforcing' ? '+' : '−'}
								</span>
								<strong>{loop.name}</strong>
							</div>
							<p class="loop-desc">{loop.description}</p>
						</div>
					{/each}
				</section>
			{/if}

			{#if relatedVars.length > 0}
				<section>
					<h3>Related Variables</h3>
					<div class="related-list">
						{#each relatedVars as v}
							<button class="related-btn" onclick={() => selectVariable(v.path)}>
								{v.name}
							</button>
						{/each}
					</div>
				</section>
			{/if}
		</div>
	</div>
{/if}

<style>
	.panel-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.3);
		z-index: 90;
	}
	.info-panel {
		position: fixed;
		top: 0;
		right: 0;
		width: 340px;
		max-width: 90vw;
		height: 100vh;
		background: var(--surface);
		border-left: 1px solid var(--border);
		z-index: 100;
		display: flex;
		flex-direction: column;
		overflow: hidden;
		animation: slide-in 0.2s ease-out;
	}
	@keyframes slide-in {
		from {
			transform: translateX(100%);
		}
		to {
			transform: translateX(0);
		}
	}
	.panel-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		padding: 16px;
		border-bottom: 1px solid var(--border);
	}
	h2 {
		font-size: 16px;
		font-weight: 600;
		color: var(--text);
		margin: 0;
	}
	.meta {
		font-size: 11px;
		color: var(--text-secondary);
	}
	.close-btn {
		background: none;
		border: none;
		color: var(--text-secondary);
		font-size: 20px;
		cursor: pointer;
		padding: 0 4px;
		line-height: 1;
	}
	.close-btn:hover {
		color: var(--text);
	}
	.panel-body {
		flex: 1;
		overflow-y: auto;
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 16px;
	}
	section h3 {
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--text-secondary);
		margin: 0 0 8px;
	}
	.description {
		font-size: 13px;
		color: var(--text);
		line-height: 1.6;
	}
	.toggle-btn {
		background: none;
		border: none;
		color: var(--accent);
		cursor: pointer;
		font-size: 13px;
		padding: 0;
		text-align: left;
	}
	.toggle-btn:hover {
		text-decoration: underline;
	}
	.expert {
		font-size: 12px;
		color: var(--text-secondary);
		line-height: 1.6;
		margin-top: 8px;
		font-family: 'SF Mono', 'Fira Code', monospace;
		white-space: pre-wrap;
	}
	.impact-card {
		display: flex;
		align-items: flex-start;
		gap: 8px;
		background: var(--surface-hover);
		border-radius: 6px;
		padding: 10px;
		margin-bottom: 6px;
	}
	.impact-arrow {
		font-size: 16px;
		font-weight: 700;
		flex-shrink: 0;
		line-height: 1.2;
	}
	.impact-card.increase .impact-arrow {
		color: #86efac;
	}
	.impact-card.decrease .impact-arrow {
		color: #fca5a5;
	}
	.impact-card p {
		font-size: 12px;
		color: var(--text);
		line-height: 1.5;
		margin: 0;
	}
	.loop-card {
		background: var(--surface-hover);
		border-radius: 6px;
		padding: 10px;
		margin-bottom: 6px;
	}
	.loop-header {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-bottom: 4px;
		font-size: 13px;
		color: var(--text);
	}
	.loop-type {
		width: 18px;
		height: 18px;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 12px;
		font-weight: 700;
		flex-shrink: 0;
	}
	.loop-type.reinforcing {
		background: #7f1d1d;
		color: #fca5a5;
	}
	.loop-type.stabilizing {
		background: #14532d;
		color: #86efac;
	}
	.loop-desc {
		font-size: 12px;
		color: var(--text-secondary);
		line-height: 1.5;
	}
	.related-list {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}
	.related-btn {
		background: var(--surface-hover);
		border: 1px solid var(--border);
		border-radius: 4px;
		padding: 4px 8px;
		font-size: 12px;
		color: var(--accent);
		cursor: pointer;
		transition: background 0.1s;
	}
	.related-btn:hover {
		background: var(--surface-active);
	}
</style>
```

**Step 2: Wire panel into layout**

Find the file where `VariableInfoPanel` is imported and rendered (likely the main page layout). Add `ParameterInfoPanel` alongside it:

```svelte
import ParameterInfoPanel from './ParameterInfoPanel.svelte';
```

And add `<ParameterInfoPanel />` next to `<VariableInfoPanel />` in the markup.

**Step 3: Run frontend check**

Run: `cd frontend && npm run check`
Expected: No type errors.

**Step 4: Commit**

```bash
git add frontend/src/components/ParameterInfoPanel.svelte
# Also add whichever layout file was modified
git commit -m "feat: add ParameterInfoPanel component"
```

---

### Task 7: Add sparkline to `ParameterInfoPanel`

**Files:**
- Modify: `frontend/src/components/ParameterInfoPanel.svelte`

**Step 1: Add sparkline section**

Import D3 and the simulation stores at the top of the script:

```typescript
import * as d3 from 'd3';
import { simulationResults } from '$lib/stores/simulation';
import { scenarios, activeScenarioIds } from '$lib/stores/scenarios';
```

Add a sparkline container ref and drawing logic:

```typescript
let sparklineEl = $state<HTMLDivElement | null>(null);

// Find the BAU scenario ID
let bauScenarioId = $derived.by(() => {
	const allScenarios = $scenarios;
	const bau = allScenarios.find((s) => s.name.toLowerCase().includes('business as usual') || s.name === 'BAU');
	return bau?.id ?? null;
});

// Get a focused scenario ID (any active non-BAU scenario, or BAU itself)
let comparisonScenarioId = $derived.by(() => {
	const active = $activeScenarioIds;
	const bau = bauScenarioId;
	for (const id of active) {
		if (id !== bau) return id;
	}
	return bau;
});

$effect(() => {
	if (!sparklineEl || !info) return;
	const fieldPath = info.impact.sparklineVariable;
	const results = $simulationResults;

	const bauData = bauScenarioId ? results.get(bauScenarioId) : null;
	const compData = comparisonScenarioId ? results.get(comparisonScenarioId) : null;

	drawSparkline(sparklineEl, fieldPath, bauData ?? null, compData ?? null);
});

function drawSparkline(
	el: HTMLDivElement,
	fieldPath: string,
	bauStates: WorldState[] | null,
	compStates: WorldState[] | null
) {
	const W = 280, H = 70;
	const m = { top: 4, right: 4, bottom: 4, left: 4 };
	const innerW = W - m.left - m.right;
	const innerH = H - m.top - m.bottom;

	function extractValues(states: WorldState[]): Array<{ year: number; value: number }> {
		return states.map((s) => {
			const [sector, field] = fieldPath.split('.');
			const val = (s as any)[sector]?.[field] ?? 0;
			return { year: s.time, value: val };
		});
	}

	const svg = d3.select(el)
		.selectAll<SVGSVGElement, null>('svg')
		.data([null])
		.join('svg')
		.attr('width', W)
		.attr('height', H);

	const g = svg
		.selectAll<SVGGElement, null>('g.spark')
		.data([null])
		.join('g')
		.attr('class', 'spark')
		.attr('transform', `translate(${m.left},${m.top})`);

	const allPoints: Array<{ year: number; value: number }> = [];
	const bauPoints = bauStates ? extractValues(bauStates) : [];
	const compPoints = compStates ? extractValues(compStates) : [];
	allPoints.push(...bauPoints, ...compPoints);

	if (allPoints.length === 0) {
		g.selectAll('*').remove();
		return;
	}

	const xScale = d3.scaleLinear()
		.domain(d3.extent(allPoints, (d) => d.year) as [number, number])
		.range([0, innerW]);

	const yScale = d3.scaleLinear()
		.domain(d3.extent(allPoints, (d) => d.value) as [number, number])
		.nice()
		.range([innerH, 0]);

	const line = d3.line<{ year: number; value: number }>()
		.x((d) => xScale(d.year))
		.y((d) => yScale(d.value));

	// BAU line (dimmed)
	g.selectAll<SVGPathElement, null>('path.bau-line')
		.data(bauPoints.length > 0 ? [bauPoints] : [])
		.join('path')
		.attr('class', 'bau-line')
		.attr('fill', 'none')
		.attr('stroke', 'var(--text-secondary)')
		.attr('stroke-width', 1)
		.attr('opacity', 0.4)
		.attr('d', (d) => line(d));

	// Current scenario line
	g.selectAll<SVGPathElement, null>('path.comp-line')
		.data(compPoints.length > 0 ? [compPoints] : [])
		.join('path')
		.attr('class', 'comp-line')
		.attr('fill', 'none')
		.attr('stroke', 'var(--accent)')
		.attr('stroke-width', 1.5)
		.attr('d', (d) => line(d));
}
```

Add to the markup, inside the Impact section after the impact cards:

```svelte
			<div class="sparkline" bind:this={sparklineEl}></div>
```

Add sparkline styles:

```css
	.sparkline {
		margin-top: 4px;
		border-radius: 4px;
		background: var(--surface-hover);
		padding: 4px;
		display: flex;
		justify-content: center;
	}
```

Also add the `WorldState` type import:

```typescript
import type { WorldState } from '$lib/types';
```

**Step 2: Run frontend check**

Run: `cd frontend && npm run check`
Expected: No type errors.

**Step 3: Commit**

```bash
git add frontend/src/components/ParameterInfoPanel.svelte
git commit -m "feat: add sparkline chart to ParameterInfoPanel"
```

---

### Task 8: Chart highlighting — dim unrelated lines

**Files:**
- Modify: `frontend/src/lib/charts/UnifiedChart.svelte:7,240-268`

**Step 1: Import `highlightedVariables` store**

Add to the imports in `UnifiedChart.svelte` (line 7):

```typescript
import { selectedVariableId } from '../stores/info';
import { highlightedVariables } from '../stores/info';
```

(Combine with existing `selectedVariableId` import on line 7.)

**Step 2: Add highlighting logic to line rendering**

In the line rendering section (lines 240-268), modify the opacity attributes. The `linesData` items have an `id` field — check what format it uses. It likely contains the `fieldPath` (e.g. `"population.population"`).

Replace the line join (lines 243-268) with:

```typescript
		const highlighted = $highlightedVariables;
		const hasHighlight = highlighted.size > 0;

		function getLineOpacity(d: LineDatum): number {
			if (!hasHighlight) return 1;
			// Extract the field path from the line ID
			// LineDatum.id format is typically "scenarioId:fieldPath"
			const fieldPath = d.id.includes(':') ? d.id.split(':').slice(1).join(':') : d.id;
			return highlighted.has(fieldPath) ? 1 : 0.15;
		}

		function getLineWidth(d: LineDatum): number {
			if (!hasHighlight) return 2;
			const fieldPath = d.id.includes(':') ? d.id.split(':').slice(1).join(':') : d.id;
			return highlighted.has(fieldPath) ? 2.5 : 2;
		}

		lines.join(
			(enter) =>
				enter
					.append('path')
					.attr('class', 'var-line')
					.attr('fill', 'none')
					.attr('stroke-width', (d) => getLineWidth(d))
					.attr('stroke', (d) => d.color)
					.attr('opacity', 0)
					.attr('d', (d) => line(d.points))
					.transition()
					.duration(400)
					.attr('opacity', (d) => getLineOpacity(d)),
			(update) =>
				update
					.transition()
					.duration(400)
					.attr('stroke', (d) => d.color)
					.attr('stroke-width', (d) => getLineWidth(d))
					.attr('opacity', (d) => getLineOpacity(d))
					.attr('d', (d) => line(d.points)),
			(exit) => exit
				.transition()
				.duration(200)
				.attr('opacity', 0)
				.remove()
		);
```

**Important:** Check the exact `LineDatum` type and `id` format first — the `fieldPath` extraction logic above assumes `id` contains the field path. Read the `LineDatum` type definition and how `linesData` is constructed to confirm.

**Step 3: Run frontend check**

Run: `cd frontend && npm run check`
Expected: No type errors.

**Step 4: Commit**

```bash
git add frontend/src/lib/charts/UnifiedChart.svelte
git commit -m "feat: dim chart lines when parameter info panel highlights related variables"
```

---

### Task 9: Integration — wire `ParameterInfoPanel` into layout

**Files:**
- Find and modify the main layout/page that renders `VariableInfoPanel`

**Step 1: Find where `VariableInfoPanel` is rendered**

Run: `grep -r "VariableInfoPanel" frontend/src/ --include="*.svelte"` to find the parent component.

**Step 2: Add `ParameterInfoPanel` alongside it**

In the same file, add:

```svelte
import ParameterInfoPanel from '../components/ParameterInfoPanel.svelte';
```

And add `<ParameterInfoPanel />` right after `<VariableInfoPanel />` in the markup.

**Step 3: Run full frontend tests**

Run: `cd frontend && npm run check && npm test`
Expected: All checks pass, all tests pass.

**Step 4: Commit**

```bash
git add <modified-layout-file>
git commit -m "feat: render ParameterInfoPanel in main layout"
```

---

### Task 10: Final verification

**Step 1: Run all backend tests**

Run: `cargo test --workspace && cargo clippy --workspace -- -D warnings`
Expected: All pass.

**Step 2: Run all frontend tests**

Run: `cd frontend && npm run check && npm test`
Expected: All pass.

**Step 3: Build frontend**

Run: `cd frontend && npm run build`
Expected: Build succeeds.

**Step 4: Manual smoke test (if dev servers available)**

Start with `./run.sh --dev`, open browser, verify:
- All 13 parameter sliders appear (including 3 new ones)
- Info icon visible on each slider
- Clicking info icon opens `ParameterInfoPanel`
- Panel shows beginner text, expert toggle, impact cards, sparkline, feedback loops, related variables
- Clicking a related variable switches to `VariableInfoPanel`
- Chart dims unrelated lines when parameter panel is open
- Escape / backdrop click closes panel

**Step 5: Final commit if any fixes needed, then done**
