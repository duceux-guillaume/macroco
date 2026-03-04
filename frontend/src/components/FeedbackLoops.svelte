<script lang="ts">
	import type { FeedbackLoopInfo } from '$lib/content/variable-descriptions';

	interface Props {
		loops: FeedbackLoopInfo[];
	}

	let { loops }: Props = $props();
</script>

{#if loops.length > 0}
	<section>
		<h3>Feedback Loops</h3>
		{#each loops as loop}
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

<style>
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
</style>
