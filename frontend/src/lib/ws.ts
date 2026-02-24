import { PUBLIC_WS_BASE } from '$env/static/public';
import { writable } from 'svelte/store';
import type { WsClientMsg, WsServerMsg } from './types';

export type ConnectionState = 'disconnected' | 'connecting' | 'connected' | 'reconnecting';

export const connectionState = writable<ConnectionState>('disconnected');

let socket: WebSocket | null = null;
let handlers: Array<(msg: WsServerMsg) => void> = [];
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
let shouldReconnect = false;

export function connect(): void {
	if (socket && (socket.readyState === WebSocket.OPEN || socket.readyState === WebSocket.CONNECTING)) {
		return;
	}

	shouldReconnect = true;
	const isReconnect = socket !== null;
	connectionState.set(isReconnect ? 'reconnecting' : 'connecting');

	socket = new WebSocket(PUBLIC_WS_BASE);

	socket.onopen = () => {
		connectionState.set('connected');
	};

	socket.onmessage = (event) => {
		try {
			const msg = JSON.parse(event.data) as WsServerMsg;
			for (const handler of handlers) {
				handler(msg);
			}
		} catch {
			console.error('Failed to parse WS message:', event.data);
		}
	};

	socket.onclose = () => {
		connectionState.set('disconnected');
		socket = null;
		if (shouldReconnect) {
			scheduleReconnect();
		}
	};

	socket.onerror = () => {
		// onclose will fire after onerror
	};
}

function scheduleReconnect(): void {
	if (reconnectTimer) return;
	connectionState.set('reconnecting');
	reconnectTimer = setTimeout(() => {
		reconnectTimer = null;
		connect();
	}, 2000);
}

export function send(msg: WsClientMsg): void {
	if (socket && socket.readyState === WebSocket.OPEN) {
		socket.send(JSON.stringify(msg));
	}
}

export function onServerMessage(handler: (msg: WsServerMsg) => void): () => void {
	handlers.push(handler);
	return () => {
		handlers = handlers.filter((h) => h !== handler);
	};
}

export function disconnect(): void {
	shouldReconnect = false;
	if (reconnectTimer) {
		clearTimeout(reconnectTimer);
		reconnectTimer = null;
	}
	if (socket) {
		socket.close();
		socket = null;
	}
	connectionState.set('disconnected');
}
