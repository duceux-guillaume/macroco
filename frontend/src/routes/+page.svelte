<script lang="ts">
	import Sidebar from '../components/Sidebar.svelte';
	import ScenarioBar from '../components/ScenarioBar.svelte';
	import ChartGrid from '../components/ChartGrid.svelte';
</script>

<svelte:head>
	<title>Macroco — World 3 Simulator</title>
</svelte:head>

<div class="app-layout">
	<Sidebar />
	<main class="main-content">
		<ScenarioBar />
		<ChartGrid />
	</main>
</div>

<style>
	.app-layout {
		display: flex;
		height: 100vh;
		overflow: hidden;
	}
	.main-content {
		flex: 1;
		display: flex;
		flex-direction: column;
		padding: 12px 16px;
		overflow-y: auto;
		min-width: 0;
	}
</style>
