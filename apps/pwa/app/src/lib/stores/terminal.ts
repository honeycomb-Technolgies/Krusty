import { writable, get } from 'svelte/store';
import { browser } from '$app/environment';

export interface TerminalTab {
	id: string;
	title: string;
	connected: boolean;
	error: string | null;
}

interface TerminalState {
	tabs: TerminalTab[];
	activeTabId: string | null;
}

const initialState: TerminalState = {
	tabs: [],
	activeTabId: null
};

export const terminalStore = writable<TerminalState>(initialState);

// Subscribe to workspace changes - cd to directory when it changes
// Imported dynamically to avoid circular dependency
let workspaceSubscribed = false;
let pendingInitialCd: string | null = null;
let workspaceUnsubscribe: (() => void) | null = null;
let workspaceSyncTimeout: ReturnType<typeof setTimeout> | null = null;

const WORKSPACE_SYNC_TIMEOUT = 30_000; // 30 seconds

export function initWorkspaceSync() {
	if (!browser || workspaceSubscribed) return;
	workspaceSubscribed = true;

	import('./workspace').then(({ workspaceStore }) => {
		let lastDir: string | null = null;
		let hasConnected = false;
		workspaceUnsubscribe = workspaceStore.subscribe((ws) => {
			if (ws.initialized && ws.directory && ws.directory !== lastDir) {
				lastDir = ws.directory;
				// cd to new directory in active terminal
				const state = get(terminalStore);
				if (state.activeTabId) {
					const tab = state.tabs.find(t => t.id === state.activeTabId);
					if (tab?.connected) {
						hasConnected = true;
						sendInput(state.activeTabId, `cd "${ws.directory}"\n`);
					} else {
						// Terminal not connected yet, queue the cd
						pendingInitialCd = ws.directory;
					}
				} else {
					// No active tab yet, queue the cd
					pendingInitialCd = ws.directory;
				}
			}
		});

		// Auto-unsubscribe after timeout if no terminal ever connected
		workspaceSyncTimeout = setTimeout(() => {
			if (!hasConnected) {
				cleanupWorkspaceSync();
			}
			workspaceSyncTimeout = null;
		}, WORKSPACE_SYNC_TIMEOUT);
	});
}

export function cleanupWorkspaceSync() {
	if (workspaceSyncTimeout) {
		clearTimeout(workspaceSyncTimeout);
		workspaceSyncTimeout = null;
	}
	workspaceUnsubscribe?.();
	workspaceUnsubscribe = null;
	workspaceSubscribed = false;
}

// Called when terminal connects - execute any pending cd
export function executePendingCd(tabId: string) {
	if (pendingInitialCd) {
		// Small delay to let terminal initialize
		setTimeout(() => {
			sendInput(tabId, `cd "${pendingInitialCd}"\n`);
			pendingInitialCd = null;
		}, 100);
	}
}

// Per-tab WebSocket connections and callbacks
const connections = new Map<string, WebSocket>();
const callbacks = new Map<string, (data: string) => void>();

let tabCounter = 0;

function generateTabId(): string {
	return `term-${++tabCounter}`;
}

export function createTab(title?: string): string {
	const id = generateTabId();
	const tab: TerminalTab = {
		id,
		title: title || `Terminal ${tabCounter}`,
		connected: false,
		error: null
	};

	terminalStore.update((s) => ({
		tabs: [...s.tabs, tab],
		activeTabId: id
	}));

	return id;
}

export function closeTab(tabId: string) {
	// Disconnect WebSocket
	const ws = connections.get(tabId);
	if (ws) {
		ws.close();
		connections.delete(tabId);
		callbacks.delete(tabId);
	}

	terminalStore.update((s) => {
		const newTabs = s.tabs.filter((t) => t.id !== tabId);
		let newActiveId = s.activeTabId;

		// If closing active tab, switch to another
		if (s.activeTabId === tabId) {
			const closedIndex = s.tabs.findIndex((t) => t.id === tabId);
			if (newTabs.length > 0) {
				// Prefer tab to the left, otherwise first tab
				newActiveId = newTabs[Math.max(0, closedIndex - 1)]?.id || newTabs[0]?.id;
			} else {
				newActiveId = null;
			}
		}

		return { tabs: newTabs, activeTabId: newActiveId };
	});
}

export function setActiveTab(tabId: string) {
	terminalStore.update((s) => ({ ...s, activeTabId: tabId }));
}

export function renameTab(tabId: string, title: string) {
	terminalStore.update((s) => ({
		...s,
		tabs: s.tabs.map((t) => (t.id === tabId ? { ...t, title } : t))
	}));
}

export function connectTerminal(tabId: string, onData: (data: string) => void) {
	// Initialize workspace sync on first connection
	initWorkspaceSync();

	// Already connected?
	const existingWs = connections.get(tabId);
	if (existingWs?.readyState === WebSocket.OPEN) {
		callbacks.set(tabId, onData);
		return;
	}

	callbacks.set(tabId, onData);

	const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
	const wsUrl = `${protocol}//${window.location.host}/ws/terminal`;

	const ws = new WebSocket(wsUrl);
	connections.set(tabId, ws);

	ws.onopen = () => {
		terminalStore.update((s) => ({
			...s,
			tabs: s.tabs.map((t) => (t.id === tabId ? { ...t, connected: true, error: null } : t))
		}));
		// Execute any pending cd from workspace initialization
		executePendingCd(tabId);
	};

	ws.onmessage = (event) => {
		const callback = callbacks.get(tabId);
		if (!callback) return;

		try {
			const msg = JSON.parse(event.data);
			if (msg.type === 'output') {
				callback(msg.data);
			}
		} catch {
			callback(event.data);
		}
	};

	ws.onerror = () => {
		terminalStore.update((s) => ({
			...s,
			tabs: s.tabs.map((t) => (t.id === tabId ? { ...t, error: 'WebSocket error' } : t))
		}));
	};

	ws.onclose = () => {
		terminalStore.update((s) => ({
			...s,
			tabs: s.tabs.map((t) => (t.id === tabId ? { ...t, connected: false } : t))
		}));
		connections.delete(tabId);
	};
}

export function disconnectTerminal(tabId: string) {
	const ws = connections.get(tabId);
	ws?.close();
	connections.delete(tabId);
	callbacks.delete(tabId);
}

export function sendInput(tabId: string, data: string) {
	const ws = connections.get(tabId);
	if (ws?.readyState === WebSocket.OPEN) {
		ws.send(JSON.stringify({ type: 'input', data }));
	}
}

export function sendResize(tabId: string, cols: number, rows: number) {
	const ws = connections.get(tabId);
	if (ws?.readyState === WebSocket.OPEN) {
		ws.send(JSON.stringify({ type: 'resize', cols, rows }));
	}
}

// Helper to get current tab's connection state
export function getTabState(tabId: string): TerminalTab | undefined {
	return get(terminalStore).tabs.find((t) => t.id === tabId);
}
