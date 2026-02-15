/**
 * Unified Workspace Store
 *
 * Single source of truth for "where am I working":
 * - currentDirectory: The project directory (null = no project open)
 * - currentSessionId: The active chat session
 *
 * Features:
 * - Cross-tab sync via BroadcastChannel
 * - Persistence via localStorage
 * - Automatic sync to IDE, Terminal, Chat
 */

import { writable, get } from 'svelte/store';
import { browser } from '$app/environment';

export interface WorkspaceState {
	/** Current working directory (null = no project, show root) */
	directory: string | null;
	/** Active session ID */
	sessionId: string | null;
	/** Whether workspace is initialized */
	initialized: boolean;
}

const STORAGE_KEY = 'krusty:workspace';
const CHANNEL_NAME = 'krusty:workspace-sync';

// Load initial state from localStorage
function loadState(): WorkspaceState {
	if (!browser) {
		return { directory: null, sessionId: null, initialized: false };
	}
	try {
		const stored = localStorage.getItem(STORAGE_KEY);
		if (stored) {
			const parsed = JSON.parse(stored);
			return { ...parsed, initialized: true };
		}
	} catch {
		// Ignore parse errors
	}
	return { directory: null, sessionId: null, initialized: true };
}

// Save state to localStorage
function saveState(state: WorkspaceState) {
	if (!browser) return;
	try {
		localStorage.setItem(STORAGE_KEY, JSON.stringify({
			directory: state.directory,
			sessionId: state.sessionId
		}));
	} catch {
		// Ignore storage errors
	}
}

// Create the store
function createWorkspaceStore() {
	const { subscribe, set, update } = writable<WorkspaceState>(loadState());

	// Cross-tab sync channel
	let channel: BroadcastChannel | null = null;
	if (browser && typeof BroadcastChannel !== 'undefined') {
		channel = new BroadcastChannel(CHANNEL_NAME);
		channel.onmessage = (event) => {
			const { type, state } = event.data;
			if (type === 'workspace-update') {
				// Update local state from other tab (don't broadcast back)
				set({ ...state, initialized: true });
			}
		};
	}

	// Broadcast state change to other tabs
	function broadcast(state: WorkspaceState) {
		if (channel) {
			channel.postMessage({ type: 'workspace-update', state });
		}
	}

	return {
		subscribe,

		/**
		 * Cleanup resources (called on app unmount)
		 */
		destroy() {
			if (channel) {
				channel.close();
				channel = null;
			}
		},

		/**
		 * Set the current workspace directory and session
		 * This is the main way to change what project is active
		 */
		setWorkspace(directory: string | null, sessionId: string | null) {
			const newState: WorkspaceState = {
				directory,
				sessionId,
				initialized: true
			};
			set(newState);
			saveState(newState);
			broadcast(newState);
		},

		/**
		 * Update just the session ID (when switching sessions in same directory)
		 */
		setSession(sessionId: string | null) {
			update(state => {
				const newState = { ...state, sessionId };
				saveState(newState);
				broadcast(newState);
				return newState;
			});
		},

		/**
		 * Update just the directory (rare - usually set both together)
		 */
		setDirectory(directory: string | null) {
			update(state => {
				const newState = { ...state, directory };
				saveState(newState);
				broadcast(newState);
				return newState;
			});
		},

		/**
		 * Clear workspace (no project open)
		 */
		clear() {
			const newState: WorkspaceState = {
				directory: null,
				sessionId: null,
				initialized: true
			};
			set(newState);
			saveState(newState);
			broadcast(newState);
		},

		/**
		 * Get current state synchronously
		 */
		getState(): WorkspaceState {
			return get({ subscribe });
		},

		/**
		 * Initialize from a session (used when loading a session)
		 */
		initFromSession(sessionId: string, directory: string | null) {
			const newState: WorkspaceState = {
				directory,
				sessionId,
				initialized: true
			};
			set(newState);
			saveState(newState);
			broadcast(newState);
		}
	};
}

export const workspaceStore = createWorkspaceStore();

// Convenience getters
export function getCurrentDirectory(): string | null {
	return workspaceStore.getState().directory;
}

export function getCurrentSessionId(): string | null {
	return workspaceStore.getState().sessionId;
}

/**
 * Validate stored workspace state on page load
 * Clears workspace if session no longer exists
 */
export async function validateWorkspace(apiClient: { getSession: (id: string) => Promise<unknown> }) {
	const state = workspaceStore.getState();
	if (!state.sessionId) return;

	try {
		await apiClient.getSession(state.sessionId);
		// Session exists, workspace is valid
	} catch {
		// Session was deleted, clear workspace
		workspaceStore.clear();
	}
}
