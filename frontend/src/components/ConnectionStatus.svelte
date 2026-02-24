<script lang="ts">
	import { connectionState } from '$lib/ws';

	const labels: Record<string, string> = {
		disconnected: 'Disconnected',
		connecting: 'Connecting...',
		connected: 'Connected',
		reconnecting: 'Reconnecting...'
	};

	const dotColors: Record<string, string> = {
		disconnected: '#ef4444',
		connecting: '#eab308',
		connected: '#22c55e',
		reconnecting: '#eab308'
	};
</script>

<div class="connection-status" title={labels[$connectionState]}>
	<span class="status-dot" style="background: {dotColors[$connectionState]}"></span>
	<span class="status-label">{labels[$connectionState]}</span>
</div>

<style>
	.connection-status {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 4px 0;
	}
	.status-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
	}
	.status-label {
		font-size: 11px;
		color: var(--text-secondary);
	}
</style>
