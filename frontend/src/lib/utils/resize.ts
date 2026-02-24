/** Svelte action that calls a callback with the element's content dimensions on resize. */
export function resize(
	node: HTMLElement,
	callback: (width: number, height: number) => void
): { destroy: () => void } {
	const observer = new ResizeObserver((entries) => {
		for (const entry of entries) {
			const { width, height } = entry.contentRect;
			callback(width, height);
		}
	});

	observer.observe(node);

	// Fire initial measurement
	const rect = node.getBoundingClientRect();
	callback(rect.width, rect.height);

	return {
		destroy() {
			observer.disconnect();
		}
	};
}
