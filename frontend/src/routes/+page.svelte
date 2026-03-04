<script lang="ts">
	import Sidebar from '../components/Sidebar.svelte';
	import ScenarioBar from '../components/ScenarioBar.svelte';
	import ChartGrid from '../components/ChartGrid.svelte';
	import VariableInfoPanel from '../components/VariableInfoPanel.svelte';
	import ParameterInfoPanel from '../components/ParameterInfoPanel.svelte';
	import HistoricalInfoPanel from '../components/HistoricalInfoPanel.svelte';

	let sidebarOpen = $state(false);
</script>

<svelte:head>
	<title>Macroco — World 3 Simulator</title>
</svelte:head>

<button
	class="hamburger"
	onclick={() => (sidebarOpen = !sidebarOpen)}
	aria-label="Toggle sidebar"
>
	<svg width="20" height="20" viewBox="0 0 20 20" fill="none">
		<line x1="3" y1="5" x2="17" y2="5" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
		<line x1="3" y1="10" x2="17" y2="10" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
		<line x1="3" y1="15" x2="17" y2="15" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
	</svg>
</button>

{#if sidebarOpen}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="drawer-backdrop" onclick={() => (sidebarOpen = false)}></div>
{/if}

<div class="app-layout">
	<div class="sidebar-drawer" class:open={sidebarOpen}>
		<Sidebar />
	</div>
	<main class="main-content">
		<ScenarioBar />
		<ChartGrid />
	</main>
	<VariableInfoPanel />
	<ParameterInfoPanel />
	<HistoricalInfoPanel />
</div>

<style>
	.app-layout {
		display: flex;
		height: 100vh;
		overflow: hidden;
	}
	.sidebar-drawer {
		display: contents;
	}
	.main-content {
		flex: 1;
		display: flex;
		flex-direction: column;
		padding: 12px 16px;
		overflow-y: auto;
		min-width: 0;
	}
	.hamburger {
		display: none;
	}
	.drawer-backdrop {
		display: none;
	}

	@media (max-width: 767px) {
		.app-layout {
			overflow-y: auto;
		}
		.sidebar-drawer {
			display: block;
			position: fixed;
			top: 0;
			left: 0;
			z-index: 80;
			transform: translateX(-100%);
			transition: transform 0.3s ease;
		}
		.sidebar-drawer.open {
			transform: translateX(0);
		}
		.hamburger {
			display: flex;
			align-items: center;
			justify-content: center;
			position: fixed;
			top: 8px;
			left: 8px;
			z-index: 70;
			width: 36px;
			height: 36px;
			border-radius: 8px;
			border: 1px solid var(--border);
			background: rgba(15, 17, 23, 0.85);
			color: var(--text);
			cursor: pointer;
			backdrop-filter: blur(4px);
		}
		.drawer-backdrop {
			display: block;
			position: fixed;
			inset: 0;
			background: rgba(0, 0, 0, 0.5);
			z-index: 75;
		}
		.main-content {
			padding: 52px 8px 8px;
		}
	}
</style>
