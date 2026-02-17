<script lang="ts">
	import { onMount } from 'svelte';
	import { ArrowLeft, Check, X, Eye, EyeOff, Loader2 } from 'lucide-svelte';
	import { apiClient, type ProviderStatus } from '$lib/api/client';

	interface Props {
		onBack: () => void;
	}

	let { onBack }: Props = $props();

	let providers = $state<ProviderStatus[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	// For editing
	let editingProvider = $state<string | null>(null);
	let apiKeyInput = $state('');
	let showApiKey = $state(false);
	let saving = $state(false);
	let apiKeyInputEl = $state<HTMLInputElement>(undefined!);

	onMount(() => {
		loadProviders();
	});

	async function loadProviders() {
		loading = true;
		error = null;
		try {
			providers = await apiClient.getCredentials();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load providers';
		} finally {
			loading = false;
		}
	}

	async function saveApiKey(providerId: string) {
		if (!apiKeyInput.trim()) return;

		saving = true;
		try {
			await apiClient.setCredential(providerId, apiKeyInput);

			providers = providers.map((p) =>
				p.id === providerId ? { ...p, configured: true } : p
			);

			editingProvider = null;
			apiKeyInput = '';
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to save';
		} finally {
			saving = false;
		}
	}

	async function removeApiKey(providerId: string) {
		try {
			await apiClient.deleteCredential(providerId);

			providers = providers.map((p) =>
				p.id === providerId ? { ...p, configured: false } : p
			);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to remove';
		}
	}

	function startEditing(providerId: string) {
		editingProvider = providerId;
		apiKeyInput = '';
		showApiKey = false;
		setTimeout(() => apiKeyInputEl?.focus(), 0);
	}

	function cancelEditing() {
		editingProvider = null;
		apiKeyInput = '';
	}
</script>

<div class="flex h-full flex-col">
	<!-- Header -->
	<div class="flex items-center gap-3 border-b border-border p-4">
		<button onclick={onBack} class="rounded-lg p-2 hover:bg-muted">
			<ArrowLeft class="h-5 w-5" />
		</button>
		<h2 class="text-lg font-semibold">AI Providers</h2>
	</div>

	<!-- Content -->
	<div class="flex-1 overflow-y-auto p-4">
		{#if loading}
			<div class="flex items-center justify-center py-8">
				<Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
			</div>
		{:else if error}
			<div class="rounded-lg bg-destructive/10 p-4 text-sm text-destructive">
				{error}
			</div>
		{:else}
			<p class="mb-4 text-sm text-muted-foreground">
				Configure API keys to access different AI model providers.
			</p>

			<div class="space-y-3">
				{#each providers as provider}
					<div class="rounded-xl border border-border bg-card p-4">
						<div class="flex items-center justify-between">
							<div class="flex items-center gap-3">
								<div
									class="flex h-8 w-8 items-center justify-center rounded-full {provider.configured
										? 'bg-green-500/20 text-green-500'
										: 'bg-muted text-muted-foreground'}"
								>
									{#if provider.configured}
										<Check class="h-4 w-4" />
									{:else}
										<X class="h-4 w-4" />
									{/if}
								</div>
								<div>
									<div class="font-medium">{provider.name}</div>
									<div class="text-xs text-muted-foreground">
										{provider.configured ? 'Configured' : 'Not configured'}
										{#if provider.has_oauth}
											<span class="ml-1 text-green-500">(OAuth)</span>
										{/if}
									</div>
								</div>
							</div>

							{#if editingProvider !== provider.id}
								<div class="flex gap-2">
									{#if provider.configured && !provider.has_oauth}
										<button
											onclick={() => removeApiKey(provider.id)}
											class="rounded-lg px-3 py-1.5 text-sm text-destructive hover:bg-destructive/10"
										>
											Remove
										</button>
									{/if}
									{#if !provider.has_oauth}
										<button
											onclick={() => startEditing(provider.id)}
											class="rounded-lg bg-muted px-3 py-1.5 text-sm hover:bg-accent"
										>
											{provider.configured ? 'Update' : 'Add Key'}
										</button>
									{/if}
								</div>
							{/if}
						</div>

						{#if editingProvider === provider.id}
							<div class="mt-4 space-y-3">
								<div class="relative">
									<input
										bind:this={apiKeyInputEl}
										type={showApiKey ? 'text' : 'password'}
										bind:value={apiKeyInput}
										placeholder="Enter API key..."
										class="w-full rounded-lg border border-border bg-background px-3 py-2 pr-10 text-sm"
									/>
									<button
										onclick={() => (showApiKey = !showApiKey)}
										class="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-muted-foreground hover:text-foreground"
									>
										{#if showApiKey}
											<EyeOff class="h-4 w-4" />
										{:else}
											<Eye class="h-4 w-4" />
										{/if}
									</button>
								</div>
								<div class="flex gap-2">
									<button
										onclick={cancelEditing}
										class="flex-1 rounded-lg border border-border px-3 py-2 text-sm hover:bg-muted"
									>
										Cancel
									</button>
									<button
										onclick={() => saveApiKey(provider.id)}
										disabled={!apiKeyInput.trim() || saving}
										class="flex-1 rounded-lg bg-primary px-3 py-2 text-sm text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
									>
										{#if saving}
											<Loader2 class="mx-auto h-4 w-4 animate-spin" />
										{:else}
											Save
										{/if}
									</button>
								</div>
							</div>
						{/if}
					</div>
				{/each}
			</div>
		{/if}
	</div>
</div>
