<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		title: string;
		meta: string;
		ariaLabel: string;
		beginner: string;
		expert: string;
		onclose: () => void;
		children: Snippet;
	}

	let { title, meta, ariaLabel, beginner, expert, onclose, children }: Props = $props();

	let showExpert = $state(false);

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') onclose();
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="panel-backdrop" onclick={onclose}></div>
<div class="info-panel" role="dialog" aria-label={ariaLabel}>
	<div class="panel-header">
		<div>
			<h2>{title}</h2>
			<span class="meta">{meta}</span>
		</div>
		<button class="close-btn" onclick={onclose} aria-label="Close panel">&times;</button>
	</div>

	<div class="panel-body">
		<section>
			<p class="description">{beginner}</p>
		</section>

		<section>
			<button class="toggle-btn" onclick={() => (showExpert = !showExpert)}>
				{showExpert ? '▾' : '▸'} Technical Detail
			</button>
			{#if showExpert}
				<p class="expert">{expert}</p>
			{/if}
		</section>

		{@render children()}
	</div>
</div>

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
	.panel-body :global(section h3) {
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
</style>
