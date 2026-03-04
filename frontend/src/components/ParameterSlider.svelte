<script lang="ts">
	import type { ParameterDescriptor } from '$lib/types';

	interface Props {
		descriptor: ParameterDescriptor;
		value: number;
		onchange: (field: string, value: number) => void;
		oninfo: (field: string) => void;
	}

	let { descriptor, value, onchange, oninfo }: Props = $props();

	function handleInput(e: Event) {
		const target = e.target as HTMLInputElement;
		onchange(descriptor.field, parseFloat(target.value));
	}

	function handleReset() {
		onchange(descriptor.field, descriptor.default);
	}
</script>

<div class="slider-row">
	<div class="slider-header">
		<div class="label-group">
			<button class="info-btn" onclick={() => oninfo(descriptor.field)} title="Learn about this parameter" aria-label={`Info about ${descriptor.label}`}>
				&#x24D8;
			</button>
			<label for={descriptor.field}>
				{descriptor.label}
			</label>
		</div>
		<span class="slider-value">
			{value.toFixed(descriptor.step < 0.01 ? 3 : descriptor.step < 0.1 ? 2 : 1)}
			<span class="unit">{descriptor.unit}</span>
		</span>
	</div>
	<div class="slider-controls">
		<input
			id={descriptor.field}
			type="range"
			min={descriptor.min}
			max={descriptor.max}
			step={descriptor.step}
			{value}
			oninput={handleInput}
		/>
		<button
			class="reset-btn"
			title="Reset to default"
			onclick={handleReset}
			disabled={value === descriptor.default}
		>
			&#x21BA;
		</button>
	</div>
</div>

<style>
	.slider-row {
		padding: 6px 0;
	}
	.slider-header {
		display: flex;
		justify-content: space-between;
		align-items: baseline;
		margin-bottom: 2px;
	}
	.label-group {
		display: flex;
		align-items: center;
		gap: 4px;
	}
	.info-btn {
		background: none;
		border: none;
		cursor: pointer;
		font-size: 14px;
		color: var(--text-secondary);
		padding: 0;
		line-height: 1;
		opacity: 0.5;
		transition: opacity 0.15s, color 0.15s;
	}
	.info-btn:hover {
		opacity: 1;
		color: var(--accent);
	}
	label {
		font-size: 12px;
		color: var(--text);
	}
	.slider-value {
		font-size: 12px;
		font-weight: 600;
		color: var(--text);
		font-variant-numeric: tabular-nums;
	}
	.unit {
		font-weight: 400;
		color: var(--text-secondary);
		font-size: 11px;
	}
	.slider-controls {
		display: flex;
		gap: 6px;
		align-items: center;
	}
	input[type='range'] {
		flex: 1;
		height: 4px;
		accent-color: var(--accent);
		cursor: pointer;
	}
	.reset-btn {
		background: none;
		border: 1px solid var(--border);
		border-radius: 4px;
		cursor: pointer;
		font-size: 14px;
		color: var(--text-secondary);
		padding: 1px 4px;
		line-height: 1;
	}
	.reset-btn:hover:not(:disabled) {
		color: var(--text);
		border-color: var(--text-secondary);
	}
	.reset-btn:disabled {
		opacity: 0.3;
		cursor: default;
	}
</style>
