<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { ArrowLeft, RefreshCw, Loader2, Clock, Terminal } from 'lucide-svelte';

	interface Props {
		onBack: () => void;
	}

	let { onBack }: Props = $props();

	interface Process {
		id: string;
		command: string;
		description: string | null;
		pid: number | null;
		status: string;
		elapsed_secs: number;
	}

	let processes = $state<Process[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let refreshInterval: ReturnType<typeof setInterval>;

	onMount(() => {
		loadProcesses();
		// Auto-refresh every 5 seconds
		refreshInterval = setInterval(loadProcesses, 5000);
	});

	onDestroy(() => {
		if (refreshInterval) clearInterval(refreshInterval);
	});

	async function loadProcesses() {
		try {
			const res = await fetch('/api/processes');
			if (!res.ok) throw new Error('Failed to load processes');
			processes = await res.json();
			error = null;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load processes';
		} finally {
			loading = false;
		}
	}

	function formatDuration(secs: number): string {
		if (secs < 60) return `${secs}s`;
		if (secs < 3600) return `${Math.floor(secs / 60)}m ${secs % 60}s`;
		const hours = Math.floor(secs / 3600);
		const mins = Math.floor((secs % 3600) / 60);
		return `${hours}h ${mins}m`;
	}

	function getStatusColor(status: string): string {
		if (status === 'Running') return 'bg-green-500/20 text-green-500';
		if (status === 'Completed') return 'bg-blue-500/20 text-blue-500';
		if (status === 'Failed') return 'bg-red-500/20 text-red-500';
		return 'bg-muted text-muted-foreground';
	}
</script>

<div class="flex h-full flex-col">
	<!-- Header -->
	<div class="flex items-center justify-between border-b border-border p-4">
		<div class="flex items-center gap-3">
			<button onclick={onBack} class="rounded-lg p-2 hover:bg-muted">
				<ArrowLeft class="h-5 w-5" />
			</button>
			<h2 class="text-lg font-semibold">Background Tasks</h2>
		</div>
		<button
			onclick={loadProcesses}
			disabled={loading}
			class="rounded-lg p-2 hover:bg-muted disabled:opacity-50"
			title="Refresh"
		>
			<RefreshCw class="h-5 w-5 {loading ? 'animate-spin' : ''}" />
		</button>
	</div>

	<!-- Content -->
	<div class="flex-1 overflow-y-auto p-4">
		{#if loading && processes.length === 0}
			<div class="flex items-center justify-center py-8">
				<Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
			</div>
		{:else if error}
			<div class="rounded-lg bg-destructive/10 p-4 text-sm text-destructive">
				{error}
			</div>
		{:else if processes.length === 0}
			<div class="py-8 text-center text-muted-foreground">
				<Terminal class="mx-auto mb-3 h-12 w-12 opacity-50" />
				<p>No background processes running</p>
			</div>
		{:else}
			<p class="mb-4 text-sm text-muted-foreground">
				{processes.filter((p) => p.status === 'Running').length} running,{' '}
				{processes.length} total
			</p>

			<div class="space-y-3">
				{#each processes as process}
					<div class="rounded-xl border border-border bg-card p-4">
						<div class="flex items-start justify-between gap-3">
							<div class="flex-1 min-w-0">
								<div class="flex items-center gap-2">
									<span
										class="inline-flex items-center rounded-full px-2 py-0.5 text-xs {getStatusColor(
											process.status
										)}"
									>
										{process.status}
									</span>
									{#if process.pid}
										<span class="text-xs text-muted-foreground">PID {process.pid}</span>
									{/if}
								</div>
								<div class="mt-2 font-mono text-sm truncate" title={process.command}>
									{process.command}
								</div>
								{#if process.description}
									<div class="mt-1 text-xs text-muted-foreground">
										{process.description}
									</div>
								{/if}
							</div>
							<div class="flex items-center gap-1 text-xs text-muted-foreground">
								<Clock class="h-3 w-3" />
								{formatDuration(process.elapsed_secs)}
							</div>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</div>
</div>
