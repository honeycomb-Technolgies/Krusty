import { get, writable } from 'svelte/store';
import {
	apiClient,
	type GitBranch,
	type GitStatusResponse,
	type GitWorktree
} from '$api/client';
import { workspaceStore } from './workspace';
import { loadSessions } from './sessions';

interface GitState {
	status: GitStatusResponse | null;
	branches: GitBranch[];
	worktrees: GitWorktree[];
	isLoading: boolean;
	error: string | null;
}

const initialState: GitState = {
	status: null,
	branches: [],
	worktrees: [],
	isLoading: false,
	error: null
};

const POLL_INTERVAL_MS = 5000;

export const gitStore = writable<GitState>(initialState);

let pollTimer: ReturnType<typeof setInterval> | null = null;
let lastDirectory: string | null = null;

workspaceStore.subscribe((ws) => {
	if (!ws.initialized) return;
	if (ws.directory === lastDirectory) return;
	lastDirectory = ws.directory;
	void refreshGit(true);
});

function currentDirectory(): string | null {
	return workspaceStore.getState().directory;
}

export async function refreshGit(forceLoading = false) {
	const directory = currentDirectory();
	if (!directory) {
		gitStore.set(initialState);
		return;
	}

	gitStore.update((s) => ({
		...s,
		isLoading: forceLoading || s.status === null,
		error: null
	}));

	try {
		const status = await apiClient.getGitStatus(directory);

		if (!status.in_repo) {
			gitStore.set({
				status,
				branches: [],
				worktrees: [],
				isLoading: false,
				error: null
			});
			return;
		}

		const [branchesRes, worktreesRes] = await Promise.all([
			apiClient.getGitBranches(directory),
			apiClient.getGitWorktrees(directory)
		]);

		gitStore.set({
			status,
			branches: branchesRes.branches,
			worktrees: worktreesRes.worktrees,
			isLoading: false,
			error: null
		});
	} catch (err) {
		gitStore.update((s) => ({
			...s,
			isLoading: false,
			error: err instanceof Error ? err.message : 'Failed to load git status'
		}));
	}
}

export async function checkoutBranch(branch: string) {
	const directory = currentDirectory();
	if (!directory) return;

	await apiClient.checkoutGitBranch(branch, directory);
	await refreshGit(false);
}

export async function switchWorktree(path: string, sessionId?: string | null) {
	workspaceStore.setDirectory(path);

	if (sessionId) {
		await apiClient.updateSession(sessionId, { working_dir: path });
		loadSessions();
	}

	await refreshGit(true);
}

export function startGitPolling() {
	if (pollTimer) return;
	pollTimer = setInterval(() => {
		const state = get(gitStore);
		if (state.isLoading) return;
		void refreshGit(false);
	}, POLL_INTERVAL_MS);
}

export function stopGitPolling() {
	if (!pollTimer) return;
	clearInterval(pollTimer);
	pollTimer = null;
}
