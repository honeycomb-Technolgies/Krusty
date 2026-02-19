<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { ArrowLeft, Check, X, Eye, EyeOff, Loader2, LogIn, LogOut } from 'lucide-svelte';
	import { apiClient, type ProviderStatus } from '$lib/api/client';

	interface Props {
		onBack: () => void;
	}

	let { onBack }: Props = $props();

	let providers = $state<ProviderStatus[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	// API key editing
	let editingProvider = $state<string | null>(null);
	let apiKeyInput = $state('');
	let showApiKey = $state(false);
	let saving = $state(false);
	let apiKeyInputEl = $state<HTMLInputElement>(undefined!);

	// OAuth state
	let oauthLoading = $state<string | null>(null);
	let pasteCodeInput = $state('');
	let pasteCodeProvider = $state<string | null>(null);
	let oauthPollingInterval = $state<ReturnType<typeof setInterval> | null>(null);
	let pasteCodeInputEl = $state<HTMLInputElement>(undefined!);

	onMount(() => {
		loadProviders();
	});

	onDestroy(() => {
		stopPolling();
	});

	function stopPolling() {
		if (oauthPollingInterval) {
			clearInterval(oauthPollingInterval);
			oauthPollingInterval = null;
		}
	}

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
		pasteCodeProvider = null;
		setTimeout(() => apiKeyInputEl?.focus(), 0);
	}

	function cancelEditing() {
		editingProvider = null;
		apiKeyInput = '';
	}

	async function startOAuth(providerId: string) {
		oauthLoading = providerId;
		error = null;
		try {
			const result = await apiClient.startOAuth(providerId);
			const popup = window.open(result.auth_url, 'krusty-oauth', 'width=600,height=700');

			if (result.paste_code) {
				// Anthropic: show paste-code input
				pasteCodeProvider = providerId;
				pasteCodeInput = '';
				editingProvider = null;
				oauthLoading = null;
				setTimeout(() => pasteCodeInputEl?.focus(), 0);
			} else {
				// OpenAI: poll for completion
				startPolling(providerId, popup);
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to start OAuth';
			oauthLoading = null;
		}
	}

	function startPolling(providerId: string, popup: Window | null) {
		stopPolling();
		oauthPollingInterval = setInterval(async () => {
			// If popup was closed by user without completing, stop after a grace period
			if (popup && popup.closed) {
				// Give the callback server a moment to process
				await new Promise((r) => setTimeout(r, 1000));
			}

			try {
				const status = await apiClient.getOAuthStatus(providerId);
				if (status.has_token) {
					stopPolling();
					oauthLoading = null;
					await loadProviders();
				} else if (!status.flow_active) {
					// Flow ended without token (timeout or error)
					stopPolling();
					oauthLoading = null;
				}
			} catch {
				// Ignore polling errors
			}
		}, 2000);
	}

	async function submitPasteCode() {
		if (!pasteCodeInput.trim() || !pasteCodeProvider) return;

		oauthLoading = pasteCodeProvider;
		error = null;
		try {
			await apiClient.exchangeOAuthCode(pasteCodeProvider, pasteCodeInput.trim());
			pasteCodeProvider = null;
			pasteCodeInput = '';
			oauthLoading = null;
			await loadProviders();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to exchange code';
			oauthLoading = null;
		}
	}

	function cancelPasteCode() {
		pasteCodeProvider = null;
		pasteCodeInput = '';
	}

	async function revokeOAuth(providerId: string) {
		try {
			await apiClient.revokeOAuth(providerId);
			providers = providers.map((p) =>
				p.id === providerId ? { ...p, has_oauth: false } : p
			);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to sign out';
		}
	}

	function statusText(provider: ProviderStatus): string {
		if (provider.has_oauth && provider.configured) return 'OAuth + API Key';
		if (provider.has_oauth) return 'OAuth connected';
		if (provider.configured) return 'API Key configured';
		return 'Not configured';
	}

	function isActive(provider: ProviderStatus): boolean {
		return provider.configured || provider.has_oauth;
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
			<div class="mb-4 rounded-lg bg-destructive/10 p-4 text-sm text-destructive">
				{error}
				<button onclick={() => (error = null)} class="ml-2 underline">dismiss</button>
			</div>
		{/if}

		{#if !loading}
			<p class="mb-4 text-sm text-muted-foreground">
				Configure API keys or sign in with OAuth to access AI providers.
			</p>

			<div class="space-y-3">
				{#each providers as provider}
					<div class="rounded-xl border border-border bg-card p-4">
						<!-- Provider header row -->
						<div class="flex items-center justify-between">
							<div class="flex items-center gap-3">
								<div
									class="flex h-8 w-8 items-center justify-center rounded-full {isActive(provider)
										? 'bg-green-500/20 text-green-500'
										: 'bg-muted text-muted-foreground'}"
								>
									{#if isActive(provider)}
										<Check class="h-4 w-4" />
									{:else}
										<X class="h-4 w-4" />
									{/if}
								</div>
								<div>
									<div class="font-medium">{provider.name}</div>
									<div class="text-xs text-muted-foreground">
										{statusText(provider)}
									</div>
								</div>
							</div>

							{#if editingProvider !== provider.id && pasteCodeProvider !== provider.id}
								<div class="flex gap-2">
									<!-- OAuth buttons -->
									{#if provider.supports_oauth}
										{#if provider.has_oauth}
											<button
												onclick={() => revokeOAuth(provider.id)}
												class="flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-sm text-destructive hover:bg-destructive/10"
											>
												<LogOut class="h-3.5 w-3.5" />
												Sign out
											</button>
										{:else}
											<button
												onclick={() => startOAuth(provider.id)}
												disabled={oauthLoading === provider.id}
												class="flex items-center gap-1.5 rounded-lg bg-primary px-3 py-1.5 text-sm text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
											>
												{#if oauthLoading === provider.id}
													<Loader2 class="h-3.5 w-3.5 animate-spin" />
													Signing in...
												{:else}
													<LogIn class="h-3.5 w-3.5" />
													Sign in
												{/if}
											</button>
										{/if}
									{/if}

									<!-- API key buttons -->
									{#if provider.configured}
										<button
											onclick={() => removeApiKey(provider.id)}
											class="rounded-lg px-3 py-1.5 text-sm text-destructive hover:bg-destructive/10"
										>
											Remove Key
										</button>
									{/if}
									<button
										onclick={() => startEditing(provider.id)}
										class="rounded-lg bg-muted px-3 py-1.5 text-sm hover:bg-accent"
									>
										{provider.configured ? 'Update Key' : 'Add Key'}
									</button>
								</div>
							{/if}
						</div>

						<!-- Paste-code input (Anthropic) -->
						{#if pasteCodeProvider === provider.id}
							<div class="mt-4 space-y-3">
								<p class="text-xs text-muted-foreground">
									Complete sign-in in the popup window, then paste the authorization code below.
								</p>
								<input
									bind:this={pasteCodeInputEl}
									type="text"
									bind:value={pasteCodeInput}
									placeholder="Paste authorization code..."
									class="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm font-mono"
								/>
								<div class="flex gap-2">
									<button
										onclick={cancelPasteCode}
										class="flex-1 rounded-lg border border-border px-3 py-2 text-sm hover:bg-muted"
									>
										Cancel
									</button>
									<button
										onclick={submitPasteCode}
										disabled={!pasteCodeInput.trim() || oauthLoading === provider.id}
										class="flex-1 rounded-lg bg-primary px-3 py-2 text-sm text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
									>
										{#if oauthLoading === provider.id}
											<Loader2 class="mx-auto h-4 w-4 animate-spin" />
										{:else}
											Submit
										{/if}
									</button>
								</div>
							</div>
						{/if}

						<!-- API key editing -->
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
