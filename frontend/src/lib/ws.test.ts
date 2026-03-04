// REQ: REQ-008
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { get } from 'svelte/store';

// Mock $env/static/public before importing ws module
vi.mock('$env/static/public', () => ({
	PUBLIC_API_BASE: 'http://localhost:8080/api/v1',
	PUBLIC_WS_BASE: 'ws://localhost:8080/api/v1/ws'
}));

// Mock WebSocket global
class MockWebSocket {
	static CONNECTING = 0;
	static OPEN = 1;
	static CLOSING = 2;
	static CLOSED = 3;

	readyState = MockWebSocket.CONNECTING;
	onopen: (() => void) | null = null;
	onclose: (() => void) | null = null;
	onmessage: ((event: { data: string }) => void) | null = null;
	onerror: (() => void) | null = null;
	url: string;
	send = vi.fn();
	close = vi.fn(() => {
		this.readyState = MockWebSocket.CLOSED;
		if (this.onclose) this.onclose();
	});

	constructor(url: string) {
		this.url = url;
		MockWebSocket.lastInstance = this;
	}

	// Simulate the server accepting the connection
	simulateOpen() {
		this.readyState = MockWebSocket.OPEN;
		if (this.onopen) this.onopen();
	}

	// Simulate receiving a message
	simulateMessage(data: string) {
		if (this.onmessage) this.onmessage({ data });
	}

	static lastInstance: MockWebSocket | null = null;
}

// Install mock before ws module loads
vi.stubGlobal('WebSocket', MockWebSocket);

// Dynamic import so the mock is in place
let ws: typeof import('./ws');
let connectionState: typeof import('./ws').connectionState;

beforeEach(async () => {
	// Reset module state by re-importing
	vi.resetModules();
	ws = await import('./ws');
	connectionState = ws.connectionState;
	MockWebSocket.lastInstance = null;
});

afterEach(() => {
	// Ensure disconnected after each test
	ws.disconnect();
});

describe('connectionState transitions', () => {
	it('goes disconnected -> connecting -> connected on connect', () => {
		expect(get(connectionState)).toBe('disconnected');

		ws.connect();
		expect(get(connectionState)).toBe('connecting');

		const socket = MockWebSocket.lastInstance!;
		socket.simulateOpen();
		expect(get(connectionState)).toBe('connected');
	});
});

describe('onServerMessage', () => {
	it('dispatches parsed messages to handlers', () => {
		ws.connect();
		const socket = MockWebSocket.lastInstance!;
		socket.simulateOpen();

		const handler = vi.fn();
		ws.onServerMessage(handler);

		const msg = { type: 'sim_complete', scenario_id: 'bau', total_steps: 100 };
		socket.simulateMessage(JSON.stringify(msg));

		expect(handler).toHaveBeenCalledOnce();
		expect(handler).toHaveBeenCalledWith(msg);
	});
});

describe('send', () => {
	it('serializes and sends when connected', () => {
		ws.connect();
		const socket = MockWebSocket.lastInstance!;
		socket.simulateOpen();

		const msg = { type: 'start_simulation' as const, scenario_id: 'bau' };
		ws.send(msg);

		expect(socket.send).toHaveBeenCalledOnce();
		expect(socket.send).toHaveBeenCalledWith(JSON.stringify(msg));
	});
});

describe('disconnect', () => {
	it('cleans up and sets state to disconnected', () => {
		ws.connect();
		const socket = MockWebSocket.lastInstance!;
		socket.simulateOpen();
		expect(get(connectionState)).toBe('connected');

		ws.disconnect();
		expect(socket.close).toHaveBeenCalled();
		expect(get(connectionState)).toBe('disconnected');
	});
});

describe('handler unsubscribe', () => {
	it('removes handler so it is not called', () => {
		ws.connect();
		const socket = MockWebSocket.lastInstance!;
		socket.simulateOpen();

		const handler = vi.fn();
		const unsub = ws.onServerMessage(handler);

		// Unsubscribe before message arrives
		unsub();

		socket.simulateMessage(JSON.stringify({ type: 'sim_error', message: 'test' }));
		expect(handler).not.toHaveBeenCalled();
	});
});
