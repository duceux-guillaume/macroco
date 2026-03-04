<script lang="ts">
	import ConnectionStatus from './ConnectionStatus.svelte';
	import SimulationControls from './SimulationControls.svelte';
	import ScenarioSelector from './ScenarioSelector.svelte';
	import ParameterSliders from './ParameterSliders.svelte';
	import { buildBugReportUrl, buildFeatureRequestUrl } from '$lib/utils/feedback-url';
	import { focusedScenario } from '$lib/stores/scenarios';

	const featureUrl = buildFeatureRequestUrl();
	let bugUrl = $derived(
		buildBugReportUrl(
			$focusedScenario?.name ?? null,
			typeof navigator !== 'undefined' ? navigator.userAgent : 'SSR'
		)
	);
</script>

<aside class="sidebar">
	<div class="sidebar-header">
		<h1>Macroco</h1>
		<p class="subtitle">World 3 Simulator</p>
		<ConnectionStatus />
	</div>

	<div class="sidebar-section">
		<SimulationControls />
	</div>

	<div class="sidebar-section">
		<ScenarioSelector />
	</div>

	<div class="sidebar-divider"></div>

	<div class="sidebar-section scrollable">
		<h3>Parameters</h3>
		<ParameterSliders />
	</div>

	<div class="sidebar-divider"></div>
	<div class="sidebar-footer">
		<a href={bugUrl} target="_blank" rel="noopener noreferrer" class="feedback-link">
			<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
				<path d="M8 2l1.88 1.88"/>
				<path d="M14.12 3.88 16 2"/>
				<path d="M9 7.13v-1a3.003 3.003 0 1 1 6 0v1"/>
				<path d="M12 20c-3.3 0-6-2.7-6-6v-3a4 4 0 0 1 4-4h4a4 4 0 0 1 4 4v3c0 3.3-2.7 6-6 6"/>
				<path d="M12 20v-9"/>
				<path d="M6.53 9C4.6 8.8 3 7.1 3 5"/>
				<path d="M6 13H2"/>
				<path d="M3 21c0-2.1 1.7-3.9 3.8-4"/>
				<path d="M20.97 5c0 2.1-1.6 3.8-3.5 4"/>
				<path d="M22 13h-4"/>
				<path d="M17.2 17c2.1.1 3.8 1.9 3.8 4"/>
			</svg>
			Report a bug
		</a>
		<a href={featureUrl} target="_blank" rel="noopener noreferrer" class="feedback-link">
			<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
				<path d="M15 14c.2-1 .7-1.7 1.5-2.5 1-.9 1.5-2.2 1.5-3.5A6 6 0 0 0 6 8c0 1 .2 2.2 1.5 3.5.7.7 1.3 1.5 1.5 2.5"/>
				<path d="M9 18h6"/>
				<path d="M10 22h4"/>
			</svg>
			Request a feature
		</a>
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
	.sidebar-footer {
		padding: 8px 16px 12px;
		display: flex;
		gap: 12px;
	}
	.feedback-link {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: 11px;
		color: var(--text-secondary);
		text-decoration: none;
		transition: color 0.15s;
	}
	.feedback-link:hover {
		color: var(--accent);
	}

	@media (max-width: 767px) {
		.sidebar {
			width: 320px;
			min-width: 320px;
		}
	}
</style>
