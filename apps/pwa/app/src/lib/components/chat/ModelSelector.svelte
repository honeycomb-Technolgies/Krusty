<script lang="ts">
	import X from 'lucide-svelte/icons/x';
	import Search from 'lucide-svelte/icons/search';
	import Brain from 'lucide-svelte/icons/brain';
	import Check from 'lucide-svelte/icons/check';
	import Clock from 'lucide-svelte/icons/clock';
	import Zap from 'lucide-svelte/icons/zap';
	import { apiClient } from '$api/client';
	import { untrack } from 'svelte';

	interface Props {
		currentModel: string;
		isOpen: boolean;
		onClose: () => void;
		onSelect: (modelId: string) => void;
	}

	let { currentModel, isOpen, onClose, onSelect }: Props = $props();

	interface Model {
		id: string;
		display_name: string;
		provider: string;
		context_window: number;
		max_output: number;
		supports_thinking: boolean;
		supports_tools: boolean;
	}

	let allModels = $state<Model[]>([]);
	let isLoading = $state(false);
	let error = $state<string | null>(null);
	let searchQuery = $state('');
	let selectedIndex = 0; // Not reactive - managed via DOM
	let searchInput = $state<HTMLInputElement>(undefined!);
	let listContainer = $state<HTMLDivElement>(undefined!);
	let scrollTimeout: ReturnType<typeof setTimeout>;

	const RECENT_KEY = 'krusty:recent_models';
	const MAX_RECENT = 5;

	function getRecentIds(): string[] {
		try {
			const stored = localStorage.getItem(RECENT_KEY);
			return stored ? JSON.parse(stored) : [];
		} catch {
			return [];
		}
	}

	function addToRecent(modelId: string) {
		const recent = getRecentIds().filter((id) => id !== modelId);
		recent.unshift(modelId);
		localStorage.setItem(RECENT_KEY, JSON.stringify(recent.slice(0, MAX_RECENT)));
	}

	// Group models by provider
	let groupedModels = $derived.by(() => {
		const query = searchQuery.toLowerCase().trim();
		const recentIds = new Set(getRecentIds());

		const filtered = query
			? allModels.filter(
					(m) =>
						m.display_name.toLowerCase().includes(query) ||
						m.id.toLowerCase().includes(query) ||
						m.provider.toLowerCase().includes(query)
				)
			: allModels;

		const recent: Model[] = [];
		const byProvider = new Map<string, Model[]>();

		for (const model of filtered) {
			if (!query && recentIds.has(model.id)) {
				recent.push(model);
			} else {
				if (!byProvider.has(model.provider)) {
					byProvider.set(model.provider, []);
				}
				byProvider.get(model.provider)!.push(model);
			}
		}

		const recentOrder = getRecentIds();
		recent.sort((a, b) => recentOrder.indexOf(a.id) - recentOrder.indexOf(b.id));

		// Sort providers: Anthropic first, then by model count (ascending)
		const sortedProviders = [...byProvider.keys()].sort((a, b) => {
			// Anthropic always first
			if (a === 'Anthropic') return -1;
			if (b === 'Anthropic') return 1;
			// Sort by model count (fewer models = higher in list)
			const countA = byProvider.get(a)!.length;
			const countB = byProvider.get(b)!.length;
			return countA - countB;
		});

		const groups: { name: string; icon: 'recent' | 'anthropic' | null; models: Model[] }[] = [];

		if (recent.length > 0) {
			groups.push({ name: 'Recent', icon: 'recent', models: recent });
		}

		for (const provider of sortedProviders) {
			groups.push({
				name: provider,
				icon: provider === 'Anthropic' ? 'anthropic' : null,
				models: byProvider.get(provider)!
			});
		}

		// Build flat index for keyboard nav
		let idx = 0;
		const modelToIndex = new Map<string, number>();
		for (const group of groups) {
			for (const model of group.models) {
				modelToIndex.set(model.id, idx++);
			}
		}

		return { groups, modelToIndex, totalModels: idx };
	});

	function formatContext(tokens: number): string {
		if (tokens >= 1000000) return `${(tokens / 1000000).toFixed(0)}M`;
		if (tokens >= 1000) return `${(tokens / 1000).toFixed(0)}K`;
		return tokens.toString();
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (!isOpen) return;

		switch (e.key) {
			case 'Escape':
				onClose();
				break;
			case 'ArrowDown':
				e.preventDefault();
				selectedIndex = Math.min(selectedIndex + 1, groupedModels.totalModels - 1);
				scrollToSelected();
				break;
			case 'ArrowUp':
				e.preventDefault();
				selectedIndex = Math.max(selectedIndex - 1, 0);
				scrollToSelected();
				break;
			case 'Enter':
				e.preventDefault();
				const el = listContainer?.querySelector(`[data-idx="${selectedIndex}"]`) as HTMLElement;
				if (el) el.click();
				break;
		}
	}

	function scrollToSelected() {
		// Update selection via DOM directly - no Svelte re-render
		listContainer?.querySelector('.selected')?.classList.remove('selected');
		const el = listContainer?.querySelector(`[data-idx="${selectedIndex}"]`) as HTMLElement;
		el?.classList.add('selected');
		el?.scrollIntoView({ block: 'nearest' });
	}

	function handleSelect(modelId: string) {
		addToRecent(modelId);
		onSelect(modelId);
		onClose();
	}

	function handleScroll() {
		listContainer?.classList.add('scrolling');
		clearTimeout(scrollTimeout);
		scrollTimeout = setTimeout(() => {
			listContainer?.classList.remove('scrolling');
		}, 150);
	}

	async function fetchModels() {
		if (allModels.length > 0) {
			const idx = groupedModels.modelToIndex.get(currentModel);
			selectedIndex = idx ?? 0;
			return;
		}

		isLoading = true;
		error = null;
		try {
			const data = await apiClient.getModels();
			allModels = data.models;
			const idx = groupedModels.modelToIndex.get(currentModel);
			selectedIndex = idx ?? 0;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load models';
		} finally {
			isLoading = false;
		}
	}

	// Only run when isOpen changes to true
	$effect(() => {
		if (isOpen) {
			// Use untrack to prevent re-running when these values change
			untrack(() => {
				searchQuery = '';
				fetchModels();
				setTimeout(() => {
					searchInput?.focus();
					const el = listContainer?.querySelector(`[data-idx="${selectedIndex}"]`);
					el?.classList.add('selected');
				}, 60);
			});
		}
	});
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if isOpen}
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 bg-black/60"
		role="presentation"
		onclick={onClose}
	></div>

	<div
		class="fixed left-1/2 top-1/2 z-[51] flex max-h-[80vh] w-full max-w-lg -translate-x-1/2 -translate-y-1/2
			flex-col rounded-xl border border-border/50 bg-card shadow-2xl"
	>
		<div class="flex shrink-0 items-center justify-between border-b border-border px-4 py-3">
			<h2 class="text-lg font-semibold">Select Model</h2>
			<div class="flex items-center gap-2 text-xs text-muted-foreground">
				<span>{allModels.length} models</span>
				<button onclick={onClose} class="rounded-lg p-1 hover:bg-muted">
					<X class="h-5 w-5" />
				</button>
			</div>
		</div>

		<div class="shrink-0 border-b border-border px-4 py-2">
			<div class="relative">
				<Search class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
				<input
					bind:this={searchInput}
					type="search"
					autocomplete="off"
					autocapitalize="off"
					spellcheck="false"
					bind:value={searchQuery}
					placeholder="Search models..."
					class="w-full rounded-lg border border-input bg-background py-2 pl-10 pr-4 text-sm
						placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
				/>
			</div>
		</div>

		<div
			bind:this={listContainer}
			onscroll={handleScroll}
			class="scroll-container min-h-0 flex-1 overflow-y-auto"
		>
			{#if isLoading}
				<div class="flex items-center justify-center py-8 text-muted-foreground">
					Loading models...
				</div>
			{:else if error}
				<div class="py-8 text-center text-destructive">{error}</div>
			{:else if groupedModels.totalModels === 0}
				<div class="py-8 text-center text-muted-foreground">No models found</div>
			{:else}
				{#each groupedModels.groups as group}
					<div class="header-item sticky top-0 z-10 flex items-center gap-2 bg-card px-4 py-1 text-xs font-semibold uppercase text-muted-foreground">
						{#if group.icon === 'recent'}
							<Clock class="h-3 w-3" />
						{:else if group.icon === 'anthropic'}
							<Zap class="h-3 w-3" />
						{/if}
						{group.name}
						<span class="font-normal">({group.models.length})</span>
					</div>
					{#each group.models as model}
						{@const idx = groupedModels.modelToIndex.get(model.id) ?? 0}
						{@const isCurrent = model.id === currentModel}
						<button
							data-idx={idx}
							onclick={() => handleSelect(model.id)}
							class="model-item mx-2 flex h-[52px] w-[calc(100%-16px)] items-center gap-3 rounded-lg px-3 text-left
								{isCurrent ? 'bg-accent/50' : ''}"
						>
							<span class="w-5 shrink-0">
								{#if isCurrent}
									<Check class="h-4 w-4 text-green-500" />
								{/if}
							</span>
							<div class="min-w-0 flex-1">
								<div class="flex items-center gap-2">
									<span class="truncate font-medium">{model.display_name}</span>
									{#if model.supports_thinking}
										<span title="Supports thinking">
											<Brain class="h-3.5 w-3.5 shrink-0 text-purple-500" />
										</span>
									{/if}
								</div>
								<div class="text-xs text-muted-foreground">
									{formatContext(model.context_window)} context
								</div>
							</div>
						</button>
					{/each}
				{/each}
			{/if}
		</div>

		<div class="shrink-0 border-t border-border px-4 py-2 text-center text-xs text-muted-foreground">
			<span class="font-medium">↑↓</span> navigate
			<span class="mx-2">·</span>
			<span class="font-medium">Enter</span> select
			<span class="mx-2">·</span>
			<span class="font-medium">Esc</span> close
		</div>
	</div>
{/if}

<style>
	.scroll-container {
		-webkit-overflow-scrolling: touch;
		overscroll-behavior: contain;
	}
	.header-item {
		contain: layout style paint;
	}
	.model-item {
		contain: layout style paint;
		content-visibility: auto;
		contain-intrinsic-size: auto 52px;
	}
	.model-item:hover {
		background-color: hsl(var(--muted));
	}
	:global(.scrolling .model-item) {
		pointer-events: none !important;
	}
</style>
