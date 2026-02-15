<script lang="ts">
	import { PanelLeftClose, PanelLeft, FolderOpen, RefreshCw, X, Save } from 'lucide-svelte';
	import FileTree from './FileTree.svelte';
	import Editor from './Editor.svelte';
	import SymbolBar from './SymbolBar.svelte';
	import DirectoryPicker from './DirectoryPicker.svelte';
	import { ideStore, loadFileTree, saveFile, closeFile, setActiveFile } from '$stores/ide';
	import { workspaceStore } from '$stores/workspace';
	import { createSession } from '$stores/sessions';
	import { goto } from '$app/navigation';

	let sidebarOpen = $state(true);
	let editorRef: Editor | null = $state(null);

	// Derived state
	let activeFile = $derived($ideStore.openFiles.find((f) => f.path === $ideStore.activeFilePath));
	let hasWorkingDir = $derived(!!$workspaceStore.directory);

	function handleSymbolInsert(symbol: string) {
		editorRef?.insertAtCursor(symbol);
	}

	function getFileName(path: string): string {
		return path.split('/').pop() || '';
	}

	function getDisplayPath(path: string | null): string {
		if (!path) return 'No folder open';
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

	function handleTabClose(e: MouseEvent, path: string) {
		e.stopPropagation();
		closeFile(path);
	}

	// Universal directory open: creates session (which updates workspaceStore, syncing everything)
	async function handleOpenProject(path: string) {
		// Create a new chat session with this directory
		// This will update workspaceStore, which triggers IDE file tree reload and terminal sync
		const session = await createSession(undefined, path);

		// Navigate to chat if session was created
		if (session) {
			goto('/');
		}
	}
</script>

<div class="flex h-full flex-col bg-background">
	{#if hasWorkingDir}
		<!-- Unified top bar -->
		<header class="flex items-center h-10 border-b border-border/50 bg-card/80 backdrop-blur-sm shrink-0">
			<!-- Sidebar toggle -->
			<button
				onclick={() => (sidebarOpen = !sidebarOpen)}
				class="flex h-10 w-10 items-center justify-center text-muted-foreground hover:text-foreground hover:bg-muted/50 transition-colors shrink-0 border-r border-border/50"
				title={sidebarOpen ? 'Hide sidebar' : 'Show sidebar'}
			>
				{#if sidebarOpen}
					<PanelLeftClose class="h-4 w-4" />
				{:else}
					<PanelLeft class="h-4 w-4" />
				{/if}
			</button>

			<!-- Path / breadcrumb -->
			<div class="flex items-center gap-2 px-3 shrink-0">
				<span class="text-xs text-muted-foreground truncate max-w-[120px] md:max-w-[200px]">
					{getDisplayPath($workspaceStore.directory)}
				</span>
				<button
					onclick={() => loadFileTree()}
					class="p-1 rounded text-muted-foreground hover:text-foreground hover:bg-muted/50 transition-colors shrink-0"
					title="Refresh files"
				>
					<RefreshCw class="h-3.5 w-3.5" />
				</button>
			</div>

			<!-- File tabs (scrollable) -->
			{#if $ideStore.openFiles.length > 0}
				<div class="flex-1 flex items-center min-w-0 border-l border-border/50 overflow-hidden">
					<div class="flex items-center overflow-x-auto scrollbar-hide">
						{#each $ideStore.openFiles as file (file.path)}
							{@const isActive = $ideStore.activeFilePath === file.path}
							<div
								onclick={() => setActiveFile(file.path)}
								onkeydown={(e) => e.key === 'Enter' && setActiveFile(file.path)}
								role="tab"
								tabindex="0"
								class="flex items-center gap-1.5 px-3 h-10 text-sm border-r border-border/30 shrink-0 transition-colors cursor-pointer
									{isActive ? 'bg-muted/50 text-foreground' : 'text-muted-foreground hover:text-foreground hover:bg-muted/30'}"
							>
								<span class="truncate max-w-[100px]">{getFileName(file.path)}</span>
								{#if file.isDirty}
									<span class="h-2 w-2 rounded-full bg-amber-500 shrink-0"></span>
								{/if}
								<button
									onclick={(e) => handleTabClose(e, file.path)}
									class="p-0.5 rounded text-muted-foreground hover:text-foreground hover:bg-muted transition-colors ml-1"
									title="Close"
								>
									<X class="h-3 w-3" />
								</button>
							</div>
						{/each}
					</div>
				</div>

				<!-- Save button -->
				<button
					onclick={() => saveFile()}
					disabled={!activeFile?.isDirty}
					class="flex items-center gap-1.5 px-3 h-10 text-sm transition-colors border-l border-border/50 shrink-0
						{activeFile?.isDirty
							? 'text-primary hover:bg-primary/10'
							: 'text-muted-foreground/50 cursor-not-allowed'}"
					title="Save (Ctrl+S)"
				>
					<Save class="h-4 w-4" />
				</button>
			{/if}
		</header>

		<!-- Main content -->
		<div class="flex flex-1 min-h-0 relative">
			<!-- File tree sidebar -->
			<aside
				class="absolute inset-y-0 left-0 z-10 w-60 transform border-r border-border/50 bg-card
					transition-transform duration-200 md:relative md:translate-x-0
					{sidebarOpen ? 'translate-x-0' : '-translate-x-full md:hidden'}"
			>
				<FileTree />
			</aside>

			<!-- Backdrop (mobile) -->
			{#if sidebarOpen}
				<button
					onclick={() => (sidebarOpen = false)}
					class="absolute inset-0 z-0 bg-black/50 md:hidden"
					aria-label="Close sidebar"
				></button>
			{/if}

			<!-- Editor area -->
			<main class="flex flex-1 flex-col min-w-0 overflow-hidden">
				{#if activeFile}
					<div class="flex-1 min-h-0">
						<Editor bind:this={editorRef} />
					</div>
					<!-- Symbol bar for mobile -->
					<div class="md:hidden">
						<SymbolBar onInsert={handleSymbolInsert} />
					</div>
				{:else}
					<div class="flex h-full flex-col items-center justify-center text-center p-4">
						<div class="mb-4 p-4 rounded-full bg-muted/30">
							<FolderOpen class="h-10 w-10 text-muted-foreground/70" />
						</div>
						<h2 class="mb-1 text-base font-medium text-foreground/80">No file open</h2>
						<p class="text-sm text-muted-foreground">
							Select a file from the sidebar
						</p>
					</div>
				{/if}
			</main>
		</div>
	{:else}
		<!-- No project open - show directory picker -->
		<div class="flex h-full items-center justify-center p-4">
			<DirectoryPicker onOpen={handleOpenProject} />
		</div>
	{/if}
</div>

<style>
	.scrollbar-hide {
		scrollbar-width: none;
		-ms-overflow-style: none;
	}
	.scrollbar-hide::-webkit-scrollbar {
		display: none;
	}
</style>
