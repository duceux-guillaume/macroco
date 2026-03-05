<script lang="ts">
	import { onDestroy } from 'svelte';
	import { selectedVariableId, selectedParameterId } from '$lib/stores/info';
	import {
		variableDescriptions,
		feedbackLoops,
		getRelatedParameters,
		type VariableInfo,
		type FeedbackLoopInfo
	} from '$lib/content/variable-descriptions';
	import InfoPanelShell from './InfoPanelShell.svelte';
	import FeedbackLoops from './FeedbackLoops.svelte';
	import RelatedVars from './RelatedVars.svelte';

	let variableId = $state<string | null>(null);

	const unsub = selectedVariableId.subscribe((v) => {
		variableId = v;
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

	let relatedParams = $derived(variableId ? getRelatedParameters(variableId) : []);

	function selectVariable(path: string) {
		selectedVariableId.set(path);
	}

	function selectParameter(key: string) {
		selectedParameterId.set(key);
	}
</script>

{#if info && variableId}
	<InfoPanelShell
		title={info.name}
		meta="{info.sector} · {info.unit}{info.isStock ? ' · Stock' : ''}"
		ariaLabel="Variable information"
		beginner={info.beginner}
		expert={info.expert}
		onclose={close}
		docPath={info?.docPath}
	>
		<FeedbackLoops loops={relatedLoops} />
		<RelatedVars vars={relatedVars} onselect={selectVariable} />
		<RelatedVars vars={relatedParams} onselect={selectParameter} title="Related Parameters" />
	</InfoPanelShell>
{/if}
