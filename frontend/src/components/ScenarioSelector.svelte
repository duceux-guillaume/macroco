<script lang="ts">
	import { scenarios, activeScenarioIds, focusedScenarioId, scenarioParamsCache } from '$lib/stores/scenarios';
	import { simulationResults } from '$lib/stores/simulation';
	import { createScenario, getScenario, runScenario } from '$lib/api';
	import type { ScenarioParams } from '$lib/types';
	import { compareMode, compareVariable } from '$lib/stores/chart-ui';
	import { unifiedVariables } from '$lib/charts/unified-config';

	let presets = $derived($scenarios.filter((s) => s.is_preset));
	let custom = $derived($scenarios.filter((s) => !s.is_preset));

	let allPresetsActive = $derived(
		presets.length > 0 && presets.every((s) => $activeScenarioIds.has(s.id))
	);

	function selectScenario(id: string) {
		focusedScenarioId.set(id);
		// Ensure active
		activeScenarioIds.update((ids) => {
			const next = new Set(ids);
			next.add(id);
			return next;
		});
		// Load params if not cached
		if (!$scenarioParamsCache.has(id)) {
			getScenario(id).then((scenario) => {
				scenarioParamsCache.update((cache) => {
					const next = new Map(cache);
					next.set(id, scenario.params);
					return next;
				});
			});
		}
	}

	function compareAll() {
		activeScenarioIds.update((ids) => {
			const next = new Set(ids);
			for (const s of presets) {
				next.add(s.id);
			}
			return next;
		});
	}

	function toggleCompare() {
		compareMode.update((v) => !v);
	}

	async function handleNewScenario() {
		const defaultParams: ScenarioParams = {
			meta: {
				id: crypto.randomUUID(),
				name: 'Custom Scenario',
				description: '',
				color_hex: '#' + Math.floor(Math.random() * 0xffffff).toString(16).padStart(6, '0'),
				created_at: new Date().toISOString()
			},
			family_planning_year: 2000,
			family_planning_efficacy: 0.75,
			health_investment_multiplier: 1.0,
			industrial_depreciation_rate: 0.05,
			service_depreciation_rate: 0.05,
			technology_growth_rate: 0.002,
			agricultural_technology: 1.0,
			agricultural_technology_growth_rate: 0.005,
			land_protection_fraction: 0.0,
			subsistence_food_per_capita: 230.0,
			resource_efficiency: 1.0,
			initial_nnr_fraction: 1.0,
			pollution_control: 0.0,
			start_year: 1900,
			end_year: 2100,
			time_step: 1.0
		};

		try {
			const scenario = await createScenario(defaultParams);
			const id = scenario.params.meta.id;

			scenarios.update((list) => [
				...list,
				{
					id,
					name: scenario.params.meta.name,
					description: scenario.params.meta.description,
					color_hex: scenario.params.meta.color_hex,
					is_preset: false
				}
			]);
			scenarioParamsCache.update((cache) => {
				const next = new Map(cache);
				next.set(id, scenario.params);
				return next;
			});

			const output = await runScenario(id);
			simulationResults.update((results) => {
				const next = new Map(results);
				next.set(id, output.states);
				return next;
			});

			selectScenario(id);
		} catch (e) {
			console.error('Failed to create scenario:', e);
		}
	}
</script>

<div class="scenario-selector">
	<div class="section-header">
		<h3>Presets</h3>
		<button
			class="compare-toggle"
			class:active={$compareMode}
			onclick={toggleCompare}
		>
			{$compareMode ? 'All vars' : 'Focus var'}
		</button>
	</div>
	<div class="scenario-list">
		{#each presets as s}
			<button
				class="scenario-btn"
				class:active={$focusedScenarioId === s.id}
				style="--color: {s.color_hex}"
				onclick={() => selectScenario(s.id)}
			>
				<span class="dot" style="background: {s.color_hex}"></span>
				<div class="scenario-info">
					<span class="scenario-name">{s.name}</span>
					{#if s.description}
						<span class="scenario-desc">{s.description}</span>
					{/if}
				</div>
			</button>
		{/each}
	</div>

	{#if $compareMode}
		<select class="variable-select" bind:value={$compareVariable}>
			{#each unifiedVariables as v (v.id)}
				<option value={v.fieldPath}>{v.label}</option>
			{/each}
		</select>
	{/if}

	{#if presets.length > 1}
		<button
			class="compare-btn"
			class:active={allPresetsActive}
			onclick={compareAll}
		>
			Compare All
		</button>
	{/if}

	{#if custom.length > 0}
		<h3>Custom</h3>
		<div class="scenario-list">
			{#each custom as s}
				<button
					class="scenario-btn"
					class:active={$focusedScenarioId === s.id}
					style="--color: {s.color_hex}"
					onclick={() => selectScenario(s.id)}
				>
					<span class="dot" style="background: {s.color_hex}"></span>
					<div class="scenario-info">
						<span class="scenario-name">{s.name}</span>
					</div>
				</button>
			{/each}
		</div>
	{/if}

	<button class="new-btn" onclick={handleNewScenario}>+ New Scenario</button>
</div>

<style>
	.scenario-selector {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	h3 {
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--text-secondary);
		margin: 0;
	}
	.scenario-list {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.scenario-btn {
		display: flex;
		align-items: flex-start;
		gap: 8px;
		padding: 6px 8px;
		border: 1px solid transparent;
		border-radius: 6px;
		background: none;
		cursor: pointer;
		font-size: 13px;
		color: var(--text);
		text-align: left;
		transition: background 0.1s;
	}
	.scenario-btn:hover {
		background: var(--surface-hover);
	}
	.scenario-btn.active {
		background: var(--surface-active);
		border-color: var(--color);
	}
	.dot {
		width: 10px;
		height: 10px;
		border-radius: 50%;
		flex-shrink: 0;
		margin-top: 3px;
	}
	.scenario-info {
		display: flex;
		flex-direction: column;
		gap: 1px;
		min-width: 0;
	}
	.scenario-name {
		font-size: 13px;
		color: var(--text);
	}
	.scenario-desc {
		font-size: 11px;
		color: var(--text-secondary);
		line-height: 1.3;
		overflow: hidden;
		text-overflow: ellipsis;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
	}
	.compare-btn {
		padding: 6px 12px;
		border: 1px solid var(--border);
		border-radius: 6px;
		background: none;
		cursor: pointer;
		font-size: 12px;
		color: var(--text-secondary);
		transition: all 0.1s;
	}
	.compare-btn:hover {
		border-color: var(--accent);
		color: var(--accent);
	}
	.compare-btn.active {
		border-color: var(--accent);
		color: var(--accent);
		background: var(--surface-active);
	}
	.new-btn {
		margin-top: 4px;
		padding: 8px 12px;
		border: 1px dashed var(--border);
		border-radius: 6px;
		background: none;
		cursor: pointer;
		font-size: 13px;
		color: var(--text-secondary);
		transition: all 0.1s;
	}
	.new-btn:hover {
		border-color: var(--accent);
		color: var(--accent);
	}
	.section-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}
	.compare-toggle {
		padding: 2px 8px;
		border: 1px solid var(--border);
		border-radius: 4px;
		background: none;
		cursor: pointer;
		font-size: 11px;
		color: var(--text-secondary);
		transition: all 0.1s;
	}
	.compare-toggle:hover {
		border-color: var(--accent);
		color: var(--accent);
	}
	.compare-toggle.active {
		background: var(--accent);
		border-color: var(--accent);
		color: var(--bg);
	}
	.variable-select {
		width: 100%;
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: 4px;
		color: var(--text);
		font-size: 12px;
		padding: 4px 8px;
		cursor: pointer;
	}
	.variable-select:focus {
		outline: 1px solid var(--accent);
		border-color: var(--accent);
	}
</style>
