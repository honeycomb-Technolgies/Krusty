<script lang="ts">
	import { onMount } from 'svelte';
	import { Folder, FolderOpen, ChevronUp, Home } from 'lucide-svelte';
	import { apiClient } from '$api/client';
	import { getLastDirectory } from '$stores/sessions';

	interface Props {
		onOpen: (path: string) => void;
	}

	let { onOpen }: Props = $props();

	interface BrowseEntry {
		name: string;
		path: string;
	}

	let currentPath = $state<string>('');
	let parentPath = $state<string | null>(null);
	let directories = $state<BrowseEntry[]>([]);
	let isLoading = $state(false);
	let error = $state<string | null>(null);

	onMount(() => {
		// Start at last used directory or home
		const lastDir = getLastDirectory();
		loadDirectory(lastDir || undefined);
	});

	async function loadDirectory(path?: string) {
		isLoading = true;
		error = null;

		try {
			const data = await apiClient.browseDirectories(path);
			currentPath = data.current;
			parentPath = data.parent;
			directories = data.directories;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load directories';
		} finally {
			isLoading = false;
		}
	}

	function handleSelect(dir: BrowseEntry) {
		loadDirectory(dir.path);
	}

	function handleGoUp() {
		if (parentPath) {
			loadDirectory(parentPath);
		}
	}

	function handleOpenHere() {
		onOpen(currentPath);
	}

	function getDisplayPath(path: string): string {
		const home = '/home/';
		if (path.startsWith(home)) {
			const afterHome = path.slice(home.length);
			const firstSlash = afterHome.indexOf('/');
			if (firstSlash > 0) {
				return '~' + afterHome.slice(firstSlash);
			}
			return '~';
		}
		return path;
	}
</script>

<div class="directory-picker">
	<div class="picker-header">
		<FolderOpen class="h-8 w-8 text-muted-foreground" />
		<h2>Open a Project</h2>
		<p>Select a directory to start editing</p>
	</div>

	<!-- Current path -->
	<div class="current-path">
		<span class="path-label">Location:</span>
		<span class="path-value">{getDisplayPath(currentPath)}</span>
	</div>

	<!-- Navigation -->
	<div class="nav-bar">
		<button
			onclick={handleGoUp}
			disabled={!parentPath}
			class="nav-btn"
			title="Go up"
		>
			<ChevronUp class="h-4 w-4" />
			<span>Up</span>
		</button>
		<button
			onclick={() => loadDirectory()}
			class="nav-btn"
			title="Go home"
		>
			<Home class="h-4 w-4" />
			<span>Home</span>
		</button>
	</div>

	<!-- Directory list -->
	<div class="dir-list">
		{#if isLoading}
			<div class="loading">Loading...</div>
		{:else if error}
			<div class="error">{error}</div>
		{:else if directories.length === 0}
			<div class="empty">No subdirectories</div>
		{:else}
			{#each directories as dir}
				<button onclick={() => handleSelect(dir)} class="dir-item">
					<Folder class="h-4 w-4 text-muted-foreground" />
					<span>{dir.name}</span>
				</button>
			{/each}
		{/if}
	</div>

	<!-- Actions -->
	<div class="actions">
		<button onclick={handleOpenHere} class="action-btn primary">
			<FolderOpen class="h-4 w-4" />
			<span>Open Project</span>
		</button>
		<p class="action-hint">Opens directory, creates chat session, and syncs terminal</p>
	</div>
</div>

<style>
	.directory-picker {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		max-width: 28rem;
		width: 100%;
		padding: 1.5rem;
		background: hsl(var(--card));
		border: 1px solid hsl(var(--border) / 0.5);
		border-radius: 1rem;
	}

	.picker-header {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.5rem;
		text-align: center;
	}

	.picker-header h2 {
		font-size: 1.125rem;
		font-weight: 600;
		margin: 0;
	}

	.picker-header p {
		font-size: 0.875rem;
		color: hsl(var(--muted-foreground));
		margin: 0;
	}

	.current-path {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem 0.75rem;
		background: hsl(var(--muted) / 0.5);
		border-radius: 0.5rem;
		font-size: 0.8125rem;
	}

	.path-label {
		color: hsl(var(--muted-foreground));
	}

	.path-value {
		font-family: ui-monospace, monospace;
		color: hsl(var(--foreground));
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.nav-bar {
		display: flex;
		gap: 0.5rem;
	}

	.nav-btn {
		display: flex;
		align-items: center;
		gap: 0.375rem;
		padding: 0.5rem 0.75rem;
		border-radius: 0.5rem;
		font-size: 0.8125rem;
		background: hsl(var(--muted));
		color: hsl(var(--foreground));
		border: 1px solid hsl(var(--border) / 0.5);
		transition: all 0.15s ease;
	}

	.nav-btn:hover:not(:disabled) {
		background: hsl(var(--accent));
	}

	.nav-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.dir-list {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		max-height: 200px;
		overflow-y: auto;
		border: 1px solid hsl(var(--border) / 0.5);
		border-radius: 0.5rem;
		padding: 0.25rem;
	}

	.dir-item {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem 0.75rem;
		border-radius: 0.375rem;
		font-size: 0.875rem;
		background: transparent;
		color: hsl(var(--foreground));
		border: none;
		text-align: left;
		transition: background 0.1s ease;
		cursor: pointer;
	}

	.dir-item:hover {
		background: hsl(var(--muted) / 0.7);
	}

	.loading, .error, .empty {
		padding: 1rem;
		text-align: center;
		font-size: 0.875rem;
		color: hsl(var(--muted-foreground));
	}

	.error {
		color: hsl(var(--destructive));
	}

	.actions {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.action-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 0.5rem;
		padding: 0.75rem 1rem;
		border-radius: 0.5rem;
		font-size: 0.875rem;
		font-weight: 500;
		border: none;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.action-btn.primary {
		background: hsl(var(--primary));
		color: hsl(var(--primary-foreground));
	}

	.action-btn.primary:hover {
		opacity: 0.9;
	}

	.action-hint {
		font-size: 0.75rem;
		color: hsl(var(--muted-foreground));
		text-align: center;
		margin: 0;
	}
</style>
