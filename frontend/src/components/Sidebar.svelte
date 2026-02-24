<script lang="ts">
	import ConnectionStatus from './ConnectionStatus.svelte';
	import ScenarioSelector from './ScenarioSelector.svelte';
	import ParameterSliders from './ParameterSliders.svelte';
</script>

<aside class="sidebar">
	<div class="sidebar-header">
		<h1>Macroco</h1>
		<p class="subtitle">World 3 Simulator</p>
		<ConnectionStatus />
	</div>

	<div class="sidebar-section">
		<ScenarioSelector />
	</div>

	<div class="sidebar-divider"></div>

	<div class="sidebar-section scrollable">
		<h3>Parameters</h3>
		<ParameterSliders />
	</div>
</aside>

<style>
	.sidebar {
		width: var(--sidebar-width);
		min-width: var(--sidebar-width);
		height: 100vh;
		display: flex;
		flex-direction: column;
		background: var(--surface);
		border-right: 1px solid var(--border);
		overflow: hidden;
	}
	.sidebar-header {
		padding: 16px 16px 8px;
	}
	h1 {
		font-size: 18px;
		font-weight: 700;
		color: var(--text);
		margin: 0;
	}
	.subtitle {
		font-size: 12px;
		color: var(--text-secondary);
		margin: 2px 0 0;
	}
	.sidebar-section {
		padding: 8px 16px;
	}
	.sidebar-section h3 {
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--text-secondary);
		margin: 0 0 8px;
	}
	.sidebar-divider {
		height: 1px;
		background: var(--border);
		margin: 0 16px;
	}
	.scrollable {
		flex: 1;
		overflow-y: auto;
		padding-bottom: 16px;
	}
</style>
