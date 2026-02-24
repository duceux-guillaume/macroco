<script lang="ts">
	import { scenarios, activeScenarioIds, focusedScenarioId } from '$lib/stores/scenarios';

	function toggleActive(id: string) {
		activeScenarioIds.update((ids) => {
			const next = new Set(ids);
			if (next.has(id)) {
				next.delete(id);
			} else {
				next.add(id);
			}
			return next;
		});
	}

	function focus(id: string) {
		focusedScenarioId.set(id);
	}
</script>

<div class="scenario-bar">
	{#each $scenarios as s}
		<button
			class="chip"
			class:active={$activeScenarioIds.has(s.id)}
			class:focused={$focusedScenarioId === s.id}
			style="--color: {s.color_hex}"
			onclick={() => toggleActive(s.id)}
			ondblclick={() => focus(s.id)}
			title="{s.name} (click: toggle, double-click: focus)"
		>
			<span class="chip-dot" style="background: {s.color_hex}"></span>
			{s.name}
		</button>
	{/each}
</div>

<style>
	.scenario-bar {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		padding: 8px 0;
	}
	.chip {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 4px 10px;
		border: 1px solid var(--border);
		border-radius: 999px;
		background: var(--surface);
		cursor: pointer;
		font-size: 12px;
		color: var(--text-secondary);
		transition: all 0.15s;
	}
	.chip:hover {
		border-color: var(--color);
	}
	.chip.active {
		background: var(--surface-active);
		border-color: var(--color);
		color: var(--text);
	}
	.chip.focused {
		box-shadow: 0 0 0 2px var(--color);
	}
	.chip-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
	}
	.chip:not(.active) .chip-dot {
		opacity: 0.4;
	}
</style>
