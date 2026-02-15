<script lang="ts">
	import { onMount } from 'svelte';
	import { slide, fade } from 'svelte/transition';
	import { flip } from 'svelte/animate';
	import { ChevronRight, ChevronDown, Folder, FolderOpen, Trash2 } from 'lucide-svelte';
	import { sessionsStore, loadSessions, loadDirectories, type Session } from '$stores/sessions';

	interface Props {
		currentSessionId: string | null;
		isCollapsed: boolean;
		onSelectSession: (sessionId: string) => void;
		onDeleteSession: (sessionId: string) => void;
		onToggleCollapse: () => void;
	}

	let { currentSessionId, isCollapsed, onSelectSession, onDeleteSession, onToggleCollapse }: Props = $props();

	// Tree node structure for sessions within a directory
	interface TreeNode {
		session: Session;
		children: TreeNode[];
		depth: number;
		isLast: boolean;
	}

	// Group sessions by working directory
	function groupByDirectory(sessions: Session[]): Map<string | null, Session[]> {
		const groups = new Map<string | null, Session[]>();

		for (const session of sessions) {
			const dir = session.working_dir ?? null;
			if (!groups.has(dir)) {
				groups.set(dir, []);
			}
			groups.get(dir)!.push(session);
		}

		return groups;
	}

	// Get short display name for directory
	function getDirectoryDisplayName(path: string | null): string {
		if (!path) return 'No Directory';
		const parts = path.split('/').filter(Boolean);
		// Show last 2 segments for readability
		return parts.slice(-2).join('/') || path;
	}

	// Build tree from flat sessions list (within a directory)
	function buildTree(sessions: Session[]): TreeNode[] {
		const sessionMap = new Map(sessions.map((s) => [s.id, s]));
		const childrenMap = new Map<string | null, Session[]>();

		// Group by parent
		for (const session of sessions) {
			const parentId = session.parent_session_id;
			// Only use parent if it exists in our list
			const effectiveParent = parentId && sessionMap.has(parentId) ? parentId : null;
			if (!childrenMap.has(effectiveParent)) {
				childrenMap.set(effectiveParent, []);
			}
			childrenMap.get(effectiveParent)!.push(session);
		}

		// Sort children by date (newest first for roots)
		for (const children of childrenMap.values()) {
			children.sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime());
		}

		// Build tree recursively
		function buildNodes(parentId: string | null, depth: number): TreeNode[] {
			const children = childrenMap.get(parentId) || [];
			return children.map((session, idx) => ({
				session,
				children: buildNodes(session.id, depth + 1),
				depth,
				isLast: idx === children.length - 1
			}));
		}

		return buildNodes(null, 0);
	}

	// Flatten tree for rendering
	function flattenTree(nodes: TreeNode[]): TreeNode[] {
		const result: TreeNode[] = [];
		function walk(node: TreeNode) {
			result.push(node);
			for (const child of node.children) {
				walk(child);
			}
		}
		for (const node of nodes) {
			walk(node);
		}
		return result;
	}

	// Format relative time
	function formatTime(dateStr: string): string {
		const date = new Date(dateStr);
		const now = new Date();
		const diff = now.getTime() - date.getTime();
		const minutes = Math.floor(diff / 60000);
		const hours = Math.floor(diff / 3600000);
		const days = Math.floor(diff / 86400000);

		if (minutes < 1) return 'now';
		if (minutes < 60) return `${minutes}m`;
		if (hours < 24) return `${hours}h`;
		if (days < 7) return `${days}d`;
		return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
	}

	// Truncate title
	function truncateTitle(title: string, maxLen: number = 22): string {
		if (title.length <= maxLen) return title;
		return title.slice(0, maxLen - 1) + '…';
	}

	// Track which directories are expanded
	let expandedDirs = $state<Set<string | null>>(new Set([null]));

	function toggleDirectory(dir: string | null) {
		const newSet = new Set(expandedDirs);
		if (newSet.has(dir)) {
			newSet.delete(dir);
		} else {
			newSet.add(dir);
		}
		expandedDirs = newSet;
	}

	// Sort directories: non-null first (alphabetically), then null
	function sortedDirectoryEntries(groups: Map<string | null, Session[]>): [string | null, Session[]][] {
		const entries = [...groups.entries()];
		return entries.sort((a, b) => {
			if (a[0] === null && b[0] === null) return 0;
			if (a[0] === null) return 1;  // null goes last
			if (b[0] === null) return -1;
			return a[0].localeCompare(b[0]);
		});
	}

	let groupedSessions = $derived(groupByDirectory($sessionsStore.sessions));
	let sortedGroups = $derived(sortedDirectoryEntries(groupedSessions));

	let hoveredId = $state<string | null>(null);

	onMount(() => {
		loadSessions();
		loadDirectories();
	});
</script>

<!-- Wrapper controls space taken - transitions width -->
<div
	class="h-full shrink-0 overflow-hidden transition-[width] duration-150 ease-out
		{isCollapsed ? 'w-10' : 'w-64'}"
>
	<!-- Panel slides via transform (GPU accelerated) -->
	<aside
		class="relative h-full w-64 border-r border-border/50 bg-card/60 transition-transform duration-150 ease-out
			{isCollapsed ? '-translate-x-[216px]' : 'translate-x-0'}"
	>
		<!-- Toggle button - desktop only (mobile uses header button) -->
		<button
			onclick={onToggleCollapse}
			class="absolute right-1 top-1 z-10 hidden h-8 w-8 items-center justify-center rounded-lg text-muted-foreground
				hover:bg-muted hover:text-foreground md:flex"
			title={isCollapsed ? 'Show sessions' : 'Hide sessions'}
		>
			<ChevronRight class="h-4 w-4 transition-transform duration-150 {isCollapsed ? '' : 'rotate-180'}" />
		</button>

		<!-- Header -->
		<div class="flex h-10 shrink-0 items-center px-3 pr-10">
			<span
				class="text-sm font-semibold text-muted-foreground transition-opacity duration-100
					{isCollapsed ? 'opacity-0' : 'opacity-100'}"
			>Sessions</span>
		</div>

		<!-- Sessions list with directory grouping -->
		<div
			class="flex-1 overflow-y-auto overflow-x-hidden transition-opacity duration-100
				{isCollapsed ? 'opacity-0' : 'opacity-100'}"
		>
			{#if $sessionsStore.isLoading}
				<div class="flex items-center justify-center py-8 text-muted-foreground">
					<span class="text-sm">Loading...</span>
				</div>
			{:else if $sessionsStore.sessions.length === 0}
				<div class="px-4 py-8 text-center text-sm text-muted-foreground">
					No sessions yet.<br />Start a conversation!
				</div>
			{:else}
				{#each sortedGroups as [dir, sessions]}
					{@const isExpanded = expandedDirs.has(dir)}
					{@const tree = buildTree(sessions)}
					{@const flatNodes = flattenTree(tree)}

					<!-- Directory header -->
					<button
						onclick={() => toggleDirectory(dir)}
						class="flex w-full items-center gap-2 px-3 py-1.5 text-xs font-medium text-muted-foreground hover:bg-muted/50"
					>
						{#if isExpanded}
							<FolderOpen class="h-3.5 w-3.5 shrink-0" />
							<ChevronDown class="h-3 w-3 shrink-0" />
						{:else}
							<Folder class="h-3.5 w-3.5 shrink-0" />
							<ChevronRight class="h-3 w-3 shrink-0" />
						{/if}
						<span class="truncate flex-1 text-left" title={dir ?? 'No Directory'}>
							{getDirectoryDisplayName(dir)}
						</span>
						<span class="shrink-0 tabular-nums">({sessions.length})</span>
					</button>

					<!-- Sessions in this directory -->
					{#if isExpanded}
						<div transition:slide={{ duration: 150 }}>
						{#each flatNodes as node (node.session.id)}
							{@const isSelected = node.session.id === currentSessionId}
							{@const isHovered = node.session.id === hoveredId}
							<div
								role="button"
								tabindex="0"
								onclick={() => onSelectSession(node.session.id)}
								onkeydown={(e) => e.key === 'Enter' && onSelectSession(node.session.id)}
								onmouseenter={() => (hoveredId = node.session.id)}
								onmouseleave={() => (hoveredId = null)}
								class="group relative flex w-full cursor-pointer items-start gap-2 px-3 py-1.5 text-left
									{isSelected ? 'bg-accent text-accent-foreground' : 'hover:bg-muted'}"
								style="padding-left: {16 + node.depth * 14}px"
								in:slide={{ duration: 150, delay: 50 }}
								animate:flip={{ duration: 150 }}
							>
								{#if node.depth > 0}
									<span class="absolute left-3 top-0 bottom-0 w-px bg-border/50"
										style="left: {12 + (node.depth - 1) * 14}px"></span>
									<span class="text-muted-foreground text-xs" style="margin-left: -2px">
										{node.isLast ? '└' : '├'}
									</span>
								{/if}

								<div class="min-w-0 flex-1">
									<div class="flex items-center justify-between gap-2">
										<span class="truncate text-sm">
											{truncateTitle(node.session.title)}
										</span>
										{#if isHovered && !isSelected}
											<button
												onclick={(e) => {
													e.stopPropagation();
													onDeleteSession(node.session.id);
												}}
												class="shrink-0 rounded p-0.5 text-muted-foreground hover:bg-destructive hover:text-destructive-foreground"
												title="Delete session"
											>
												<Trash2 class="h-3 w-3" />
											</button>
										{:else}
											<span class="shrink-0 text-xs text-muted-foreground">
												{formatTime(node.session.updated_at)}
											</span>
										{/if}
									</div>
									{#if node.children.length > 0}
										<div class="text-xs text-muted-foreground">
											{node.children.length} branch{node.children.length !== 1 ? 'es' : ''}
										</div>
									{/if}
								</div>
							</div>
						{/each}
						</div>
					{/if}
				{/each}
			{/if}
		</div>
	</aside>
</div>
