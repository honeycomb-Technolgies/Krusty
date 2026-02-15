<script lang="ts">
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import TerminalTabs from './TerminalTabs.svelte';
	import TerminalInstance from './TerminalInstance.svelte';
	import QuickActions from './QuickActions.svelte';
	import { terminalStore, createTab, sendInput } from '$stores/terminal';

	// Track terminal instance refs by tabId
	let instanceRefs: Record<string, TerminalInstance> = {};

	onMount(() => {
		if (!browser) return;

		// Create initial tab if none exist
		if ($terminalStore.tabs.length === 0) {
			createTab();
		}
	});

	function handleTabChange(tabId: string) {
		// Focus the new active terminal
		setTimeout(() => {
			instanceRefs[tabId]?.focus();
		}, 50);
	}

	function handleQuickAction(command: string) {
		const activeTabId = $terminalStore.activeTabId;
		if (activeTabId) {
			sendInput(activeTabId, command);
		}
	}
</script>

<div class="terminal-wrapper">
	<!-- Tab bar -->
	<TerminalTabs onTabChange={handleTabChange} />

	<!-- Terminal instances -->
	<div class="terminals-container">
		{#each $terminalStore.tabs as tab (tab.id)}
			<TerminalInstance
				tabId={tab.id}
				isActive={$terminalStore.activeTabId === tab.id}
				bind:this={instanceRefs[tab.id]}
			/>
		{/each}

		{#if $terminalStore.tabs.length === 0}
			<div class="no-terminals">
				<p>No terminals open</p>
				<p class="hint">Click + to create a new terminal</p>
			</div>
		{/if}
	</div>

	<!-- Quick actions bar (mobile-friendly) -->
	<QuickActions onAction={handleQuickAction} />
</div>

<style>
	.terminal-wrapper {
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 0;
		background: #0a0a0a;
	}

	.terminals-container {
		flex: 1 1 0;
		min-height: 200px;
		position: relative;
		overflow: hidden;
	}

	.no-terminals {
		height: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 0.5rem;
		color: hsl(var(--muted-foreground));
	}

	.no-terminals p {
		margin: 0;
	}

	.no-terminals .hint {
		font-size: 0.875rem;
		opacity: 0.7;
	}
</style>
