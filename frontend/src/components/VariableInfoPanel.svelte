<script lang="ts">
	import { onDestroy } from 'svelte';
	import { selectedVariableId } from '$lib/stores/info';
	import {
		variableDescriptions,
		feedbackLoops,
		type VariableInfo,
		type FeedbackLoopInfo
	} from '$lib/content/variable-descriptions';
	import InfoPanelShell from './InfoPanelShell.svelte';

	let showExpert = $state(false);
	let variableId = $state<string | null>(null);

	const unsub = selectedVariableId.subscribe((v) => {
		variableId = v;
		showExpert = false;
	});
	onDestroy(unsub);

	let info = $derived<VariableInfo | null>(variableId ? variableDescriptions[variableId] ?? null : null);

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
		selectedVariableId.set(null);
	}

	function selectVariable(path: string) {
		selectedVariableId.set(path);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') close();
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if info && variableId}
	<InfoPanelShell
		title={info.name}
		meta="{info.sector} · {info.unit}{info.isStock ? ' · Stock' : ''}"
		ariaLabel="Variable information"
		onclose={close}
	>
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
	</InfoPanelShell>
{/if}

<style>
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
