<script lang="ts">
	import { ArrowRight, Smartphone, Server, Download } from 'lucide-svelte';
</script>

<svelte:head>
	<title>Krusty App</title>
	<meta
		name="description"
		content="Self-hosted Krusty app for chat and workspace access from phone and desktop."
	/>
</svelte:head>

<section class="relative flex min-h-dvh items-center justify-center px-6">
	<div class="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_20%_20%,rgba(44,173,255,0.18),transparent_30%),radial-gradient(circle_at_80%_70%,rgba(255,174,0,0.12),transparent_35%)]"></div>

	<div class="relative z-10 w-full max-w-3xl rounded-3xl border border-border/60 bg-card/60 p-8 backdrop-blur-sm">
		<p class="mb-3 text-xs uppercase tracking-[0.22em] text-muted-foreground">Self-Hosted App</p>
		<h1 class="font-mono text-4xl leading-tight text-foreground md:text-5xl">
			Krusty PWA
		</h1>
		<p class="mt-4 max-w-2xl text-sm text-muted-foreground md:text-base">
			This build is focused on local-first chat and workspace access. Run your own
			`krusty-server` and connect from mobile or desktop.
		</p>

		<div class="mt-8 grid gap-3 md:grid-cols-3">
			<div class="rounded-xl border border-border/60 bg-background/50 p-4">
				<Server class="h-5 w-5 text-primary" />
				<p class="mt-2 text-sm font-medium">Server Endpoint</p>
				<p class="text-xs text-muted-foreground">Configure `VITE_API_BASE` to point at your server.</p>
			</div>
			<div class="rounded-xl border border-border/60 bg-background/50 p-4">
				<Smartphone class="h-5 w-5 text-primary" />
				<p class="mt-2 text-sm font-medium">Mobile Ready</p>
				<p class="text-xs text-muted-foreground">Optimized layout and installable manifest for phones.</p>
			</div>
			<div class="rounded-xl border border-border/60 bg-background/50 p-4">
				<Download class="h-5 w-5 text-primary" />
				<p class="mt-2 text-sm font-medium">Desktop Wrapper</p>
				<p class="text-xs text-muted-foreground">Same app surface can be wrapped as desktop shell.</p>
			</div>
		</div>

		<div class="mt-8 flex flex-wrap gap-3">
			<a
				href="/app"
				class="inline-flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
			>
				Open App
				<ArrowRight class="h-4 w-4" />
			</a>
			<a
				href="https://github.com/BurgessTG/Krusty"
				target="_blank"
				rel="noreferrer"
				class="inline-flex items-center rounded-lg border border-border px-4 py-2 text-sm text-foreground hover:bg-muted"
			>
				Project Repository
			</a>
		</div>
	</div>
</section>
