<script lang="ts">
	import { onMount } from 'svelte';
	import { ArrowLeft, Check, X, RefreshCw, Loader2, ChevronDown, ChevronRight } from 'lucide-svelte';

	interface Props {
		onBack: () => void;
	}

	let { onBack }: Props = $props();

	interface McpTool {
		name: string;
		description: string | null;
	}

	interface McpServer {
		name: string;
		server_type: string;
		status: string;
		connected: boolean;
		tool_count: number;
		tools: McpTool[];
		error: string | null;
	}

	let servers = $state<McpServer[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let expandedServer = $state<string | null>(null);
	let connecting = $state<string | null>(null);

	onMount(() => {
		loadServers();
	});

	async function loadServers() {
		loading = true;
		error = null;
		try {
			const res = await fetch('/api/mcp');
			if (!res.ok) throw new Error('Failed to load MCP servers');
			servers = await res.json();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load servers';
		} finally {
			loading = false;
		}
	}

	async function reloadConfig() {
		loading = true;
		try {
			const res = await fetch('/api/mcp/reload', { method: 'POST' });
			if (!res.ok) throw new Error('Failed to reload config');
			servers = await res.json();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to reload';
		} finally {
			loading = false;
		}
	}

	async function connectServer(name: string) {
		connecting = name;
		try {
			const res = await fetch(`/api/mcp/${name}/connect`, { method: 'POST' });
			if (!res.ok) throw new Error('Failed to connect');
			const updated = await res.json();
			servers = servers.map((s) => (s.name === name ? updated : s));
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to connect';
		} finally {
			connecting = null;
		}
	}

	async function disconnectServer(name: string) {
		connecting = name;
		try {
			const res = await fetch(`/api/mcp/${name}/disconnect`, { method: 'POST' });
			if (!res.ok) throw new Error('Failed to disconnect');
			const updated = await res.json();
			servers = servers.map((s) => (s.name === name ? updated : s));
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to disconnect';
		} finally {
			connecting = null;
		}
	}

	function toggleExpanded(name: string) {
		expandedServer = expandedServer === name ? null : name;
	}
</script>

<div class="flex h-full flex-col">
	<!-- Header -->
	<div class="flex items-center justify-between border-b border-border p-4">
		<div class="flex items-center gap-3">
			<button onclick={onBack} class="rounded-lg p-2 hover:bg-muted">
				<ArrowLeft class="h-5 w-5" />
			</button>
			<h2 class="text-lg font-semibold">MCP Servers</h2>
		</div>
		<button
			onclick={reloadConfig}
			disabled={loading}
			class="rounded-lg p-2 hover:bg-muted disabled:opacity-50"
			title="Reload config"
		>
			<RefreshCw class="h-5 w-5 {loading ? 'animate-spin' : ''}" />
		</button>
	</div>

	<!-- Content -->
	<div class="flex-1 overflow-y-auto p-4">
		{#if loading && servers.length === 0}
			<div class="flex items-center justify-center py-8">
				<Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
			</div>
		{:else if error}
			<div class="rounded-lg bg-destructive/10 p-4 text-sm text-destructive">
				{error}
			</div>
		{:else if servers.length === 0}
			<div class="py-8 text-center text-muted-foreground">
				<p class="mb-2">No MCP servers configured</p>
				<p class="text-sm">Add servers to <code class="rounded bg-muted px-1">.mcp.json</code></p>
			</div>
		{:else}
			<p class="mb-4 text-sm text-muted-foreground">
				Model Context Protocol servers provide additional tools for the AI.
			</p>

			<div class="space-y-3">
				{#each servers as server}
					<div class="rounded-xl border border-border bg-card">
						<!-- Server header -->
						<div class="flex items-center justify-between p-4">
							<button
								onclick={() => toggleExpanded(server.name)}
								class="flex flex-1 items-center gap-3 text-left"
							>
								<div
									class="flex h-8 w-8 items-center justify-center rounded-full {server.connected
										? 'bg-green-500/20 text-green-500'
										: server.error
											? 'bg-red-500/20 text-red-500'
											: 'bg-muted text-muted-foreground'}"
								>
									{#if server.connected}
										<Check class="h-4 w-4" />
									{:else if server.error}
										<X class="h-4 w-4" />
									{:else}
										<X class="h-4 w-4" />
									{/if}
								</div>
								<div class="flex-1">
									<div class="font-medium">{server.name}</div>
									<div class="text-xs text-muted-foreground">
										{server.server_type} · {server.tool_count} tools
										{#if server.error}
											<span class="text-red-500"> · {server.error}</span>
										{/if}
									</div>
								</div>
								{#if server.tools.length > 0}
									{#if expandedServer === server.name}
										<ChevronDown class="h-4 w-4 text-muted-foreground" />
									{:else}
										<ChevronRight class="h-4 w-4 text-muted-foreground" />
									{/if}
								{/if}
							</button>

							{#if server.server_type === 'stdio'}
								<button
									onclick={() =>
										server.connected ? disconnectServer(server.name) : connectServer(server.name)}
									disabled={connecting === server.name}
									class="rounded-lg px-3 py-1.5 text-sm {server.connected
										? 'text-destructive hover:bg-destructive/10'
										: 'bg-muted hover:bg-accent'}"
								>
									{#if connecting === server.name}
										<Loader2 class="h-4 w-4 animate-spin" />
									{:else if server.connected}
										Disconnect
									{:else}
										Connect
									{/if}
								</button>
							{/if}
						</div>

						<!-- Tools list (expanded) -->
						{#if expandedServer === server.name && server.tools.length > 0}
							<div class="border-t border-border px-4 py-3">
								<div class="text-xs font-medium text-muted-foreground mb-2">Available Tools</div>
								<div class="space-y-2">
									{#each server.tools as tool}
										<div class="rounded-lg bg-muted/50 p-2">
											<div class="text-sm font-mono">{tool.name}</div>
											{#if tool.description}
												<div class="text-xs text-muted-foreground mt-0.5">{tool.description}</div>
											{/if}
										</div>
									{/each}
								</div>
							</div>
						{/if}
					</div>
				{/each}
			</div>
		{/if}
	</div>
</div>
