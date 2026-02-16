<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import {
		Laptop,
		Globe,
		RefreshCw,
		Pin,
		PinOff,
		EyeOff,
		ExternalLink,
		Copy,
		AlertCircle
	} from 'lucide-svelte';

	import { apiClient, getApiUrl, type PortEntry, type PreviewSettings } from '$api/client';

	let ports = $state<PortEntry[]>([]);
	let settings = $state<PreviewSettings | null>(null);
	let loading = $state(true);
	let refreshing = $state(false);
	let mutatingPort = $state<number | null>(null);
	let error = $state<string | null>(null);
	let discoveryWarning = $state<string | null>(null);
	let copiedPort = $state<number | null>(null);
	let pollTimer: ReturnType<typeof setInterval> | null = null;
	let inFlight = false;

	onMount(() => {
		void loadPorts();
	});

	onDestroy(() => {
		clearPollTimer();
	});

	function clearPollTimer() {
		if (pollTimer) {
			clearInterval(pollTimer);
			pollTimer = null;
		}
	}

	function resetPollTimer(intervalSecs: number) {
		clearPollTimer();
		const safeInterval = Math.max(2, intervalSecs || 5);
		pollTimer = setInterval(() => {
			void loadPorts(true);
		}, safeInterval * 1000);
	}

	async function loadPorts(background = false) {
		if (inFlight && background) return;
		inFlight = true;
		if (!background) {
			loading = true;
		} else {
			refreshing = true;
		}

		try {
			const response = await apiClient.getPorts();
			ports = response.ports;
			settings = response.settings;
			discoveryWarning = response.discovery_error ?? null;
			error = null;
			resetPollTimer(response.settings.auto_refresh_secs);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load preview ports';
		} finally {
			inFlight = false;
			loading = false;
			refreshing = false;
		}
	}

	function getPreviewUrl(port: number): string {
		return getApiUrl(`/ports/${port}/proxy`);
	}

	async function copyPreviewUrl(port: number) {
		try {
			await navigator.clipboard.writeText(getPreviewUrl(port));
			copiedPort = port;
			setTimeout(() => {
				if (copiedPort === port) copiedPort = null;
			}, 1500);
		} catch {
			error = 'Failed to copy preview URL';
		}
	}

	async function togglePin(port: PortEntry) {
		mutatingPort = port.port;
		try {
			if (port.pinned) {
				await apiClient.removePinnedPort(port.port);
			} else {
				await apiClient.addPinnedPort(port.port);
			}
			await loadPorts(true);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to update pinned ports';
		} finally {
			mutatingPort = null;
		}
	}

	async function hidePort(port: number) {
		mutatingPort = port;
		try {
			await apiClient.addHiddenPort(port);
			await loadPorts(true);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to hide port';
		} finally {
			mutatingPort = null;
		}
	}
</script>

<div class="flex h-full flex-col bg-background">
	<div class="flex items-center gap-2 border-b border-border bg-card px-4 py-2">
		<Laptop class="h-4 w-4 text-muted-foreground" />
		<span class="text-sm font-medium">Port Forwarding</span>
		<div class="flex-1"></div>
		<button
			onclick={() => loadPorts(true)}
			class="rounded-lg bg-muted p-2 text-muted-foreground hover:text-foreground"
			disabled={refreshing}
			title="Refresh previews"
		>
			<RefreshCw class="h-4 w-4 {refreshing ? 'animate-spin' : ''}" />
		</button>
	</div>

	<div class="flex-1 overflow-y-auto p-4 md:p-6">
		{#if loading}
			<div class="flex h-full items-center justify-center text-muted-foreground">
				<RefreshCw class="h-5 w-5 animate-spin" />
			</div>
		{:else if settings && !settings.enabled}
			<div class="mx-auto max-w-xl rounded-xl border border-border bg-card p-5 text-center">
				<div class="mx-auto mb-3 flex h-12 w-12 items-center justify-center rounded-full bg-muted">
					<Globe class="h-6 w-6 text-muted-foreground" />
				</div>
				<h2 class="text-lg font-semibold">Preview Forwarding Disabled</h2>
				<p class="mt-2 text-sm text-muted-foreground">
					Enable preview forwarding in Menu → Settings → Preview & Port Forwarding.
				</p>
			</div>
		{:else}
			{#if error}
				<div class="mb-4 flex items-start gap-2 rounded-lg bg-destructive/10 p-3 text-sm text-destructive">
					<AlertCircle class="mt-0.5 h-4 w-4 shrink-0" />
					<span>{error}</span>
				</div>
			{/if}

			{#if discoveryWarning}
				<div class="mb-4 flex items-start gap-2 rounded-lg bg-amber-500/10 p-3 text-sm text-amber-700 dark:text-amber-400">
					<AlertCircle class="mt-0.5 h-4 w-4 shrink-0" />
					<span>{discoveryWarning}</span>
				</div>
			{/if}

			{#if ports.length === 0}
				<div class="mx-auto max-w-xl rounded-xl border border-border bg-card p-5 text-center">
					<div class="mx-auto mb-3 flex h-12 w-12 items-center justify-center rounded-full bg-muted">
						<Globe class="h-6 w-6 text-muted-foreground" />
					</div>
					<h2 class="text-lg font-semibold">No Preview Servers Detected</h2>
					<p class="mt-2 text-sm text-muted-foreground">
						Start a dev server in Terminal or pin a port from Settings. Common ports like
						`3000` and `5173` are detected automatically.
					</p>
				</div>
			{:else}
				<div class="mb-3 text-xs text-muted-foreground">
					{ports.filter((p) => p.active).length} active · {ports.length} visible
				</div>

				<div class="grid gap-3">
					{#each ports as port}
						<div class="rounded-xl border border-border bg-card p-4">
							<div class="flex flex-wrap items-center gap-2">
								<span class="font-medium">{port.name}</span>
								<span class="rounded-full bg-muted px-2 py-0.5 text-xs text-muted-foreground">
									localhost:{port.port}
								</span>
								<span
									class="rounded-full px-2 py-0.5 text-xs {port.active
										? 'bg-green-500/15 text-green-600 dark:text-green-400'
										: 'bg-amber-500/15 text-amber-700 dark:text-amber-400'}"
								>
									{port.active ? 'Active' : 'Pinned (offline)'}
								</span>
								{#if port.pinned}
									<span class="rounded-full bg-primary/15 px-2 py-0.5 text-xs text-primary">
										Pinned
									</span>
								{/if}
							</div>

							{#if port.description}
								<div class="mt-2 text-sm text-muted-foreground">{port.description}</div>
							{/if}
							{#if port.command}
								<div class="mt-1 truncate font-mono text-xs text-muted-foreground" title={port.command}>
									{port.command}
								</div>
							{/if}

							<div class="mt-3 flex flex-wrap gap-2">
								<a
									href={getPreviewUrl(port.port)}
									target="_blank"
									rel="noreferrer"
									class="inline-flex items-center gap-1 rounded-lg border border-border px-3 py-1.5 text-sm hover:bg-muted {port.active
										? ''
										: 'pointer-events-none opacity-50'}"
									title={port.active ? 'Open preview' : 'Port is not currently listening'}
								>
									<ExternalLink class="h-3.5 w-3.5" />
									Open
								</a>

								<button
									onclick={() => copyPreviewUrl(port.port)}
									class="inline-flex items-center gap-1 rounded-lg border border-border px-3 py-1.5 text-sm hover:bg-muted"
								>
									<Copy class="h-3.5 w-3.5" />
									{copiedPort === port.port ? 'Copied' : 'Copy URL'}
								</button>

								<button
									onclick={() => togglePin(port)}
									disabled={mutatingPort === port.port}
									class="inline-flex items-center gap-1 rounded-lg border border-border px-3 py-1.5 text-sm hover:bg-muted disabled:opacity-50"
								>
									{#if port.pinned}
										<PinOff class="h-3.5 w-3.5" />
										Unpin
									{:else}
										<Pin class="h-3.5 w-3.5" />
										Pin
									{/if}
								</button>

								<button
									onclick={() => hidePort(port.port)}
									disabled={mutatingPort === port.port}
									class="inline-flex items-center gap-1 rounded-lg border border-border px-3 py-1.5 text-sm hover:bg-muted disabled:opacity-50"
								>
									<EyeOff class="h-3.5 w-3.5" />
									Hide
								</button>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		{/if}
	</div>
</div>
