import { PUBLIC_API_BASE, PUBLIC_WS_BASE } from '$env/static/public';

/**
 * Returns the API base URL. In production (same origin), PUBLIC_API_BASE is
 * empty so we fall back to the relative path '/api/v1'.
 */
export function getApiBase(): string {
	return PUBLIC_API_BASE || '/api/v1';
}

/**
 * Returns the WebSocket URL. In production (same origin), PUBLIC_WS_BASE is
 * empty so we derive the URL from the current page location.
 */
export function getWsBase(): string {
	if (PUBLIC_WS_BASE) {
		return PUBLIC_WS_BASE;
	}
	const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
	return `${proto}//${window.location.host}/api/v1/ws`;
}
