<script lang="ts">
	interface Props {
		vars: Array<{ path: string; name: string }>;
		onselect: (path: string) => void;
		title?: string;
	}

	let { vars, onselect, title = 'Related Variables' }: Props = $props();
</script>

{#if vars.length > 0}
	<section>
		<h3>{title}</h3>
		<div class="related-list">
			{#each vars as v}
				<button class="related-btn" onclick={() => onselect(v.path)}>
					{v.name}
				</button>
			{/each}
		</div>
	</section>
{/if}

<style>
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
