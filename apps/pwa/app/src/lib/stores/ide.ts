import { writable, get } from 'svelte/store';
import { apiClient, type TreeEntry } from '$api/client';
import { terminalStore, sendInput } from './terminal';
import { workspaceStore } from './workspace';
import { browser } from '$app/environment';

export interface TreeNode {
	name: string;
	path: string;
	isDir: boolean;
	children?: TreeNode[];
}

export interface OpenFile {
	path: string;
	content: string;
	isDirty: boolean;
}

interface IDEState {
	tree: TreeNode[];
	openFiles: OpenFile[];
	activeFilePath: string | null;
	isLoading: boolean;
	error: string | null;
}

const initialState: IDEState = {
	tree: [],
	openFiles: [],
	activeFilePath: null,
	isLoading: false,
	error: null
};

export const ideStore = writable<IDEState>(initialState);

// Subscribe to workspace changes - sync file tree when directory changes
let workspaceUnsubscribe: (() => void) | null = null;

if (browser) {
	let lastDir: string | null = null;
	workspaceUnsubscribe = workspaceStore.subscribe((ws) => {
		if (ws.initialized && ws.directory !== lastDir) {
			lastDir = ws.directory;
			if (ws.directory) {
				loadFileTree(ws.directory);
			} else {
				// No directory - clear the tree
				ideStore.update((s) => ({ ...s, tree: [], openFiles: [], activeFilePath: null }));
			}
		}
	});
}

// Transform API response (snake_case) to frontend (camelCase)
function transformTreeNode(node: TreeEntry): TreeNode {
	return {
		name: node.name,
		path: node.path,
		isDir: node.is_dir,
		children: node.children?.map(transformTreeNode)
	};
}

export function setWorkingDir(dir: string | null) {
	// Update workspace store - this will trigger the subscription above
	workspaceStore.setDirectory(dir);
}

// Track if we're currently loading to prevent infinite loops
let isLoadingTree = false;

export async function loadFileTree(root?: string) {
	// Prevent re-entry from workspace subscription
	if (isLoadingTree) return;
	isLoadingTree = true;

	ideStore.update((s) => ({ ...s, isLoading: true, error: null }));

	try {
		const wsState = workspaceStore.getState();
		const treeRoot = root || wsState.directory || undefined;
		const data = await apiClient.getFileTree(treeRoot);

		ideStore.update((s) => ({
			...s,
			tree: data.entries.map(transformTreeNode),
			isLoading: false
		}));

		// Only update workspace if explicitly loading a new root (not from subscription)
		// This prevents infinite loops
		if (root && data.root !== wsState.directory) {
			workspaceStore.setDirectory(data.root);
		}
	} catch (err) {
		ideStore.update((s) => ({
			...s,
			isLoading: false,
			error: err instanceof Error ? err.message : 'Failed to load file tree'
		}));
	} finally {
		isLoadingTree = false;
	}
}

export async function openFile(path: string, syncTerminal = true) {
	const state = get(ideStore);

	// Check if already open
	const existing = state.openFiles.find((f) => f.path === path);
	if (existing) {
		ideStore.update((s) => ({ ...s, activeFilePath: path }));
		return;
	}

	ideStore.update((s) => ({ ...s, isLoading: true, error: null }));

	try {
		const data = await apiClient.getFile(path);

		ideStore.update((s) => ({
			...s,
			openFiles: [...s.openFiles, { path, content: data.content, isDirty: false }],
			activeFilePath: path,
			isLoading: false
		}));

		// Note: Opening a file does NOT change the workspace directory
		// The workspace is tied to the session, not individual files
		// Users can open files from anywhere without changing their project context

		if (syncTerminal) {
			syncTerminalToDirectory();
		}
	} catch (err) {
		ideStore.update((s) => ({
			...s,
			isLoading: false,
			error: err instanceof Error ? err.message : 'Failed to open file'
		}));
	}
}

export function setActiveFile(path: string) {
	ideStore.update((s) => ({ ...s, activeFilePath: path }));
}

export async function saveFile(path?: string) {
	const state = get(ideStore);
	const targetPath = path || state.activeFilePath;
	if (!targetPath) return;

	const file = state.openFiles.find((f) => f.path === targetPath);
	if (!file || !file.isDirty) return;

	ideStore.update((s) => ({ ...s, isLoading: true }));

	try {
		await apiClient.writeFile(targetPath, file.content);
		ideStore.update((s) => ({
			...s,
			openFiles: s.openFiles.map((f) =>
				f.path === targetPath ? { ...f, isDirty: false } : f
			),
			isLoading: false
		}));
	} catch (err) {
		ideStore.update((s) => ({
			...s,
			isLoading: false,
			error: err instanceof Error ? err.message : 'Failed to save file'
		}));
	}
}

export function updateFileContent(path: string, content: string) {
	ideStore.update((s) => ({
		...s,
		openFiles: s.openFiles.map((f) =>
			f.path === path ? { ...f, content, isDirty: true } : f
		)
	}));
}

export function closeFile(path?: string) {
	const state = get(ideStore);
	const targetPath = path || state.activeFilePath;
	if (!targetPath) return;

	const idx = state.openFiles.findIndex((f) => f.path === targetPath);
	if (idx === -1) return;

	ideStore.update((s) => {
		const newOpenFiles = s.openFiles.filter((f) => f.path !== targetPath);
		let newActivePath = s.activeFilePath;

		if (s.activeFilePath === targetPath) {
			// Switch to adjacent tab or null
			if (newOpenFiles.length > 0) {
				newActivePath = newOpenFiles[Math.min(idx, newOpenFiles.length - 1)]?.path || null;
			} else {
				newActivePath = null;
			}
		}

		return {
			...s,
			openFiles: newOpenFiles,
			activeFilePath: newActivePath
		};
	});
}

// Get active file's content and dirty state (for Editor component)
export function getActiveFile(): OpenFile | null {
	const state = get(ideStore);
	if (!state.activeFilePath) return null;
	return state.openFiles.find((f) => f.path === state.activeFilePath) || null;
}

// Sync terminal to current workspace directory
export function syncTerminalToDirectory() {
	const wsState = workspaceStore.getState();
	const termState = get(terminalStore);

	if (!wsState.directory || !termState.activeTabId) return;

	sendInput(termState.activeTabId, `cd "${wsState.directory}"\n`);
}

// Get current working directory from workspace store
export function getWorkingDir(): string | null {
	return workspaceStore.getState().directory;
}

export function cleanupIde() {
	workspaceUnsubscribe?.();
	workspaceUnsubscribe = null;
}
