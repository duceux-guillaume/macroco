<script lang="ts">
	import { scenarios, activeScenarioIds, focusedScenarioId, scenarioParamsCache } from '$lib/stores/scenarios';
	import { simulationResults } from '$lib/stores/simulation';
	import { createScenario, getScenario, runScenario } from '$lib/api';
	import type { ScenarioParams } from '$lib/types';

	let presets = $derived($scenarios.filter((s) => s.is_preset));
	let custom = $derived($scenarios.filter((s) => !s.is_preset));

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
			investment_rate: 0.12,
			agricultural_technology: 1.0,
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

			// Add to stores
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

			// Run simulation
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
	<h3>Presets</h3>
	<div class="scenario-list">
		{#each presets as s}
			<button
				class="scenario-btn"
				class:active={$focusedScenarioId === s.id}
				style="--color: {s.color_hex}"
				onclick={() => selectScenario(s.id)}
			>
				<span class="dot" style="background: {s.color_hex}"></span>
				{s.name}
			</button>
		{/each}
	</div>

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
					{s.name}
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
		align-items: center;
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
</style>
