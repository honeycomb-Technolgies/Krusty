<script lang="ts">
	import { Plus, X } from 'lucide-svelte';
	import { terminalStore, createTab, closeTab, setActiveTab, type TerminalTab } from '$stores/terminal';

	interface Props {
		onTabChange?: (tabId: string) => void;
	}

	let { onTabChange }: Props = $props();

	function handleNewTab() {
		const id = createTab();
		onTabChange?.(id);
	}

	function handleSelectTab(tabId: string) {
		setActiveTab(tabId);
		onTabChange?.(tabId);
	}

	function handleCloseTab(e: MouseEvent, tabId: string) {
		e.stopPropagation();
		closeTab(tabId);
	}
</script>

<div class="terminal-tabs">
	<div class="tabs-scroll">
		{#each $terminalStore.tabs as tab (tab.id)}
			<div
				onclick={() => handleSelectTab(tab.id)}
				onkeydown={(e) => e.key === 'Enter' && handleSelectTab(tab.id)}
				role="tab"
				tabindex="0"
				class="tab"
				class:active={$terminalStore.activeTabId === tab.id}
			>
				<span class="tab-status" class:connected={tab.connected} class:error={tab.error}></span>
				<span class="tab-title">{tab.title}</span>
				<button
					onclick={(e) => handleCloseTab(e, tab.id)}
					class="tab-close"
					title="Close terminal"
				>
					<X class="h-3 w-3" />
				</button>
			</div>
		{/each}
	</div>

	<button onclick={handleNewTab} class="new-tab" title="New terminal">
		<Plus class="h-4 w-4" />
	</button>
</div>

<style>
	.terminal-tabs {
		display: flex;
		align-items: center;
		gap: 0.25rem;
		padding: 0.375rem 0.5rem;
		background: hsl(var(--card) / 0.8);
		border-bottom: 1px solid hsl(var(--border) / 0.5);
		overflow: hidden;
	}

	.tabs-scroll {
		display: flex;
		gap: 0.25rem;
		overflow-x: auto;
		flex: 1;
		scrollbar-width: none;
	}

	.tabs-scroll::-webkit-scrollbar {
		display: none;
	}

	.tab {
		display: flex;
		align-items: center;
		gap: 0.375rem;
		padding: 0.375rem 0.5rem;
		border-radius: 0.375rem;
		font-size: 0.75rem;
		font-weight: 500;
		color: hsl(var(--muted-foreground));
		background: transparent;
		border: 1px solid transparent;
		white-space: nowrap;
		transition: all 0.15s ease;
		flex-shrink: 0;
		cursor: pointer;
		user-select: none;
	}

	.tab:hover {
		background: hsl(var(--muted) / 0.5);
		color: hsl(var(--foreground));
	}

	.tab.active {
		background: hsl(var(--muted));
		color: hsl(var(--foreground));
		border-color: hsl(var(--border) / 0.5);
	}

	.tab-status {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: hsl(var(--muted-foreground) / 0.5);
		flex-shrink: 0;
	}

	.tab-status.connected {
		background: hsl(142 71% 45%);
	}

	.tab-status.error {
		background: hsl(var(--destructive));
	}

	.tab-title {
		max-width: 100px;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.tab-close {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 0.125rem;
		border-radius: 0.25rem;
		color: hsl(var(--muted-foreground));
		opacity: 0;
		transition: all 0.15s ease;
	}

	.tab:hover .tab-close {
		opacity: 1;
	}

	.tab-close:hover {
		background: hsl(var(--destructive) / 0.2);
		color: hsl(var(--destructive));
	}

	.new-tab {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 1.75rem;
		height: 1.75rem;
		border-radius: 0.375rem;
		color: hsl(var(--muted-foreground));
		background: transparent;
		border: 1px dashed hsl(var(--border) / 0.5);
		flex-shrink: 0;
		transition: all 0.15s ease;
	}

	.new-tab:hover {
		background: hsl(var(--muted) / 0.5);
		color: hsl(var(--foreground));
		border-style: solid;
	}
</style>
