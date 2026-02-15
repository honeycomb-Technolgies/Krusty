<script lang="ts">
	import { Laptop, Globe, RefreshCw } from 'lucide-svelte';

	// Port forwarding - shows dev servers running on user's machine
	let ports = $state<{ port: number; name: string }[]>([]);
	let loading = $state(false);
	let connected = $state(false);

	async function refresh() {
		loading = true;
		// Fetch from /api/ports when CLI is connected
		await new Promise(r => setTimeout(r, 500));
		loading = false;
	}
</script>

<div class="flex h-full flex-col bg-background">
	<div class="flex items-center gap-2 border-b border-border bg-card px-4 py-2">
		<Laptop class="h-4 w-4 text-muted-foreground" />
		<span class="text-sm font-medium">Port Forwarding</span>
		<div class="flex-1"></div>
		<button
			onclick={refresh}
			class="rounded-lg bg-muted p-2 text-muted-foreground hover:text-foreground"
			disabled={loading}
		>
			<RefreshCw class="h-4 w-4 {loading ? 'animate-spin' : ''}" />
		</button>
	</div>

	<div class="flex-1 flex items-center justify-center p-8">
		{#if !connected}
			<div class="max-w-md text-center">
				<div class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-muted">
					<Globe class="h-8 w-8 text-muted-foreground" />
				</div>
				<h2 class="text-xl font-bold">Preview Dev Servers</h2>
				<p class="mt-2 text-muted-foreground">
					When your CLI is connected, you'll see dev servers running on your machine here.
					Access localhost:3000, localhost:5173, etc. from anywhere.
				</p>
				<p class="mt-4 text-sm text-muted-foreground">
					Run <code class="rounded bg-muted px-2 py-1">krusty login</code> on your machine to connect.
				</p>
			</div>
		{:else if ports.length === 0}
			<div class="text-center text-muted-foreground">
				<p>No dev servers detected.</p>
				<p class="text-sm mt-2">Start a dev server on your machine.</p>
			</div>
		{:else}
			<div class="grid gap-4 w-full max-w-2xl">
				{#each ports as { port, name }}
					<a
						href="/api/ports/{port}/proxy"
						target="_blank"
						class="flex items-center gap-4 rounded-xl border border-border bg-card p-4 hover:bg-muted transition-colors"
					>
						<div class="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
							<Globe class="h-5 w-5 text-primary" />
						</div>
						<div class="flex-1">
							<div class="font-medium">{name || `Port ${port}`}</div>
							<div class="text-sm text-muted-foreground">localhost:{port}</div>
						</div>
					</a>
				{/each}
			</div>
		{/if}
	</div>
</div>
