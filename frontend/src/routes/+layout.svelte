<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { getParamsSchema, getScenarios, runScenario, getScenario } from '$lib/api';
	import { paramsSchema } from '$lib/stores/schema';
	import { scenarios, activeScenarioIds, focusedScenarioId, scenarioParamsCache } from '$lib/stores/scenarios';
	import { simulationResults, streamingBuffer, simulatingScenarioId } from '$lib/stores/simulation';
	import { connect, disconnect, onServerMessage } from '$lib/ws';
	import type { WsServerMsg, WorldState } from '$lib/types';
	import '../app.css';

	interface Props {
		children: import('svelte').Snippet;
	}

	let { children }: Props = $props();

	let unsubWs: (() => void) | null = null;

	function handleWsMessage(msg: WsServerMsg) {
		switch (msg.type) {
			case 'sim_step': {
				const scenarioId = $simulatingScenarioId;
				if (!scenarioId) break;
				streamingBuffer.update((buf) => {
					const next = new Map(buf);
					const states = next.get(scenarioId) ?? [];
					states.push(msg.state);
					next.set(scenarioId, states);
					return next;
				});
				break;
			}
			case 'sim_complete': {
				// Flush streaming buffer to completed results
				const sid = msg.scenario_id;
				const buf = $streamingBuffer.get(sid);
				if (buf && buf.length > 0) {
					simulationResults.update((results) => {
						const next = new Map(results);
						next.set(sid, [...buf]);
						return next;
					});
				}
				streamingBuffer.update((b) => {
					const next = new Map(b);
					next.delete(sid);
					return next;
				});
				simulatingScenarioId.set(null);
				break;
			}
			case 'sim_error': {
				console.error('Simulation error:', msg.message);
				simulatingScenarioId.set(null);
				break;
			}
			case 'params_ack': {
				// Server acknowledged param update; simulation will restart automatically
				simulatingScenarioId.set(msg.scenario_id);
				// Clear streaming buffer for fresh results
				streamingBuffer.update((buf) => {
					const next = new Map(buf);
					next.set(msg.scenario_id, []);
					return next;
				});
				break;
			}
		}
	}

	onMount(async () => {
		// 1. Fetch schema
		try {
			const schema = await getParamsSchema();
			paramsSchema.set(schema);
		} catch (e) {
			console.error('Failed to load parameter schema:', e);
		}

		// 2. Fetch scenarios (presets are auto-created on server start)
		try {
			const scenarioList = await getScenarios();
			scenarios.set(scenarioList);

			// Activate all presets by default
			const presetIds = new Set(scenarioList.filter((s) => s.is_preset).map((s) => s.id));
			activeScenarioIds.set(presetIds);

			// Focus the first preset
			const firstPreset = scenarioList.find((s) => s.is_preset);
			if (firstPreset) {
				focusedScenarioId.set(firstPreset.id);
			}

			// Load params and run simulations for each preset via REST
			for (const s of scenarioList.filter((s) => s.is_preset)) {
				try {
					const [scenario, output] = await Promise.all([
						getScenario(s.id),
						runScenario(s.id)
					]);
					scenarioParamsCache.update((cache) => {
						const next = new Map(cache);
						next.set(s.id, scenario.params);
						return next;
					});
					simulationResults.update((results) => {
						const next = new Map(results);
						next.set(s.id, output.states);
						return next;
					});
				} catch (e) {
					console.error(`Failed to load/run scenario ${s.name}:`, e);
				}
			}
		} catch (e) {
			console.error('Failed to load scenarios:', e);
		}

		// 3. Connect WebSocket
		connect();
		unsubWs = onServerMessage(handleWsMessage);
	});

	onDestroy(() => {
		if (unsubWs) unsubWs();
		disconnect();
	});
</script>

{@render children()}
