<script lang="ts">
	import { onMount } from 'svelte';
	import { ChevronRight, ChevronDown, File, Folder, FolderOpen, RefreshCw } from 'lucide-svelte';
	import { ideStore, loadFileTree, openFile, type TreeNode } from '$stores/ide';

	let expandedDirs = $state<Set<string>>(new Set());

	function toggleDir(path: string) {
		if (expandedDirs.has(path)) {
			expandedDirs.delete(path);
		} else {
			expandedDirs.add(path);
		}
		expandedDirs = new Set(expandedDirs);
	}

	function handleFileClick(node: TreeNode) {
		if (node.isDir) {
			toggleDir(node.path);
		} else {
			openFile(node.path);
		}
	}

	function getFileExtension(name: string): string {
		return name.split('.').pop()?.toLowerCase() || '';
	}

	function getFileColor(ext: string): string {
		const colors: Record<string, string> = {
			ts: 'text-blue-400',
			tsx: 'text-blue-400',
			js: 'text-yellow-400',
			jsx: 'text-yellow-400',
			svelte: 'text-orange-500',
			vue: 'text-green-500',
			html: 'text-orange-400',
			css: 'text-blue-300',
			scss: 'text-pink-400',
			json: 'text-yellow-300',
			md: 'text-gray-400',
			rs: 'text-orange-600',
			py: 'text-blue-500',
			go: 'text-cyan-400',
			toml: 'text-gray-500',
			yaml: 'text-red-400',
			yml: 'text-red-400'
		};
		return colors[ext] || 'text-gray-400';
	}

	onMount(() => {
		loadFileTree();
	});
</script>

<div class="flex h-full flex-col">
	<!-- Tree -->
	<div class="flex-1 overflow-y-auto overflow-x-hidden p-1 pt-2">
		{#if $ideStore.isLoading && $ideStore.tree.length === 0}
			<div class="flex items-center justify-center py-8">
				<RefreshCw class="h-5 w-5 animate-spin text-muted-foreground" />
			</div>
		{:else if $ideStore.error}
			<div class="p-4 text-center">
				<p class="text-sm text-destructive">{$ideStore.error}</p>
				<button
					onclick={() => loadFileTree()}
					class="mt-2 text-xs text-primary hover:underline"
				>
					Try again
				</button>
			</div>
		{:else if $ideStore.tree.length === 0}
			<p class="py-4 text-center text-sm text-muted-foreground">No files found</p>
		{:else}
			{#each $ideStore.tree as node}
				{@render treeNode(node, 0)}
			{/each}
		{/if}
	</div>
</div>

{#snippet treeNode(node: TreeNode, depth: number)}
	{@const ext = getFileExtension(node.name)}
	{@const isExpanded = expandedDirs.has(node.path)}
	{@const isSelected = $ideStore.activeFilePath === node.path}
	<div>
		<button
			onclick={() => handleFileClick(node)}
			class="flex w-full items-center gap-1 rounded px-1.5 py-0.5 text-left text-sm transition-colors
				{isSelected ? 'bg-primary/15 text-primary' : 'text-foreground/70 hover:bg-muted hover:text-foreground'}"
			style="padding-left: {depth * 12 + 6}px"
		>
			{#if node.isDir}
				<span class="shrink-0 w-4 h-4 flex items-center justify-center">
					{#if isExpanded}
						<ChevronDown class="h-3 w-3" />
					{:else}
						<ChevronRight class="h-3 w-3" />
					{/if}
				</span>
				<span class="shrink-0 text-amber-500">
					{#if isExpanded}
						<FolderOpen class="h-4 w-4" />
					{:else}
						<Folder class="h-4 w-4" />
					{/if}
				</span>
			{:else}
				<span class="w-4"></span>
				<span class="shrink-0 {getFileColor(ext)}">
					<File class="h-4 w-4" />
				</span>
			{/if}
			<span class="truncate">{node.name}</span>
		</button>

		{#if node.isDir && isExpanded && node.children}
			{#each node.children as child}
				{@render treeNode(child, depth + 1)}
			{/each}
		{/if}
	</div>
{/snippet}

<style>
	/* Custom scrollbar */
	.overflow-y-auto {
		scrollbar-width: thin;
		scrollbar-color: hsl(var(--muted)) transparent;
	}

	.overflow-y-auto::-webkit-scrollbar {
		width: 6px;
	}

	.overflow-y-auto::-webkit-scrollbar-track {
		background: transparent;
	}

	.overflow-y-auto::-webkit-scrollbar-thumb {
		background: hsl(var(--muted));
		border-radius: 3px;
	}
</style>
