<script lang="ts">
	import { onMount } from 'svelte';
	import { ArrowLeft, Bell, Loader2, Server, TestTube2, Monitor, Pin, EyeOff, Save, Trash2 } from 'lucide-svelte';
	import { apiClient, type PreviewSettings, type PushStatusResponse } from '$lib/api/client';
	import {
		getCurrentPushState,
		isPushSupported,
		reconcilePushSubscription,
		subscribeToPush,
		type PushState,
		unsubscribeFromPush
	} from '$lib/push';

	interface Props {
		onBack: () => void;
	}

	let { onBack }: Props = $props();

	let notifications = $state(false);
	let initializing = $state(true);
	let subscribing = $state(false);
	let pushError = $state('');
	let pushSupported = $state(false);
	let pushPermission = $state<NotificationPermission | 'unsupported'>('unsupported');
	let pushEndpoint = $state<string | null>(null);
	let pushStatus = $state<PushStatusResponse | null>(null);
	let statusError = $state('');
	let testingPush = $state(false);
	let testMessage = $state('');
	let serverUrl = $state('http://localhost:3000');
	let previewSettings = $state<PreviewSettings | null>(null);
	let previewLoading = $state(true);
	let previewSaving = $state(false);
	let previewError = $state('');
	let previewMessage = $state('');
	let blockedPortsInput = $state('');
	let newPinnedPort = $state('');
	let newHiddenPort = $state('');

	onMount(() => {
		void Promise.all([initializePushSettings(), loadPreviewSettings()]);
	});

	function applyPushState(state: PushState) {
		notifications = state.subscribed;
		pushSupported = state.supported;
		pushPermission = state.permission;
		pushEndpoint = state.endpoint;
	}

	async function refreshPushStatus() {
		statusError = '';
		try {
			pushStatus = await apiClient.getPushStatus();
		} catch (e) {
			pushStatus = null;
			statusError = e instanceof Error ? e.message : 'Failed to load push status';
		}
	}

	async function initializePushSettings() {
		initializing = true;
		pushError = '';
		try {
			applyPushState(await reconcilePushSubscription());
		} catch (e) {
			pushError = e instanceof Error ? e.message : 'Failed to initialize push notifications';
			try {
				applyPushState(await getCurrentPushState());
			} catch {
				pushSupported = isPushSupported();
			}
		}
		await refreshPushStatus();
		initializing = false;
	}

	async function toggleNotifications() {
		if (subscribing || initializing) return;
		subscribing = true;
		pushError = '';
		testMessage = '';

		try {
			if (notifications) {
				await unsubscribeFromPush();
			} else {
				const ok = await subscribeToPush();
				if (!ok) {
					pushError =
						Notification.permission === 'denied'
							? 'Permission denied in browser settings'
							: 'Permission not granted';
				}
			}
			applyPushState(await getCurrentPushState());
			await refreshPushStatus();
		} catch (e) {
			pushError = e instanceof Error ? e.message : 'Failed';
			try {
				applyPushState(await getCurrentPushState());
			} catch {
				// ignore
			}
		} finally {
			subscribing = false;
		}
	}

	async function sendTestNotification() {
		if (testingPush) return;
		testingPush = true;
		testMessage = '';
		pushError = '';

		try {
			const result = await apiClient.sendPushTest();
			if (result.sent > 0) {
				testMessage = `Sent to ${result.sent}/${result.attempted} subscription${result.sent === 1 ? '' : 's'}.`;
			} else if (result.attempted === 0) {
				testMessage = 'No active subscriptions found on the server.';
			} else {
				testMessage = `No successful deliveries (${result.failed} failed, ${result.stale_removed} stale removed).`;
			}
			await refreshPushStatus();
		} catch (e) {
			testMessage = e instanceof Error ? e.message : 'Failed to send test notification';
		} finally {
			testingPush = false;
		}
	}

	function toPortList(input: string): number[] {
		return input
			.split(',')
			.map((part) => Number(part.trim()))
			.filter((port) => Number.isInteger(port) && port > 0 && port <= 65535);
	}

	function syncBlockedPortsInput(settings: PreviewSettings) {
		blockedPortsInput = settings.blocked_ports.join(', ');
	}

	async function loadPreviewSettings() {
		previewLoading = true;
		previewError = '';
		try {
			previewSettings = await apiClient.getPreviewSettings();
			syncBlockedPortsInput(previewSettings);
		} catch (e) {
			previewError = e instanceof Error ? e.message : 'Failed to load preview settings';
		} finally {
			previewLoading = false;
		}
	}

	async function savePreviewSettings() {
		if (!previewSettings) return;
		previewSaving = true;
		previewError = '';
		previewMessage = '';
		try {
			previewSettings = await apiClient.updatePreviewSettings({
				enabled: previewSettings.enabled,
				auto_refresh_secs: previewSettings.auto_refresh_secs,
				show_only_http_like: previewSettings.show_only_http_like,
				blocked_ports: toPortList(blockedPortsInput)
			});
			syncBlockedPortsInput(previewSettings);
			previewMessage = 'Preview settings saved';
		} catch (e) {
			previewError = e instanceof Error ? e.message : 'Failed to save preview settings';
		} finally {
			previewSaving = false;
		}
	}

	async function addPinnedPort() {
		const port = Number(newPinnedPort.trim());
		if (!Number.isInteger(port) || port <= 0 || port > 65535) {
			previewError = 'Pinned port must be between 1 and 65535';
			return;
		}
		previewSaving = true;
		previewError = '';
		try {
			previewSettings = await apiClient.addPinnedPort(port);
			newPinnedPort = '';
			previewMessage = `Pinned port ${port}`;
		} catch (e) {
			previewError = e instanceof Error ? e.message : 'Failed to pin port';
		} finally {
			previewSaving = false;
		}
	}

	async function removePinnedPort(port: number) {
		previewSaving = true;
		previewError = '';
		try {
			previewSettings = await apiClient.removePinnedPort(port);
			previewMessage = `Removed pinned port ${port}`;
		} catch (e) {
			previewError = e instanceof Error ? e.message : 'Failed to remove pinned port';
		} finally {
			previewSaving = false;
		}
	}

	async function addHiddenPort() {
		const port = Number(newHiddenPort.trim());
		if (!Number.isInteger(port) || port <= 0 || port > 65535) {
			previewError = 'Hidden port must be between 1 and 65535';
			return;
		}
		previewSaving = true;
		previewError = '';
		try {
			previewSettings = await apiClient.addHiddenPort(port);
			newHiddenPort = '';
			previewMessage = `Hidden port ${port}`;
		} catch (e) {
			previewError = e instanceof Error ? e.message : 'Failed to hide port';
		} finally {
			previewSaving = false;
		}
	}

	async function removeHiddenPort(port: number) {
		previewSaving = true;
		previewError = '';
		try {
			previewSettings = await apiClient.removeHiddenPort(port);
			previewMessage = `Unhidden port ${port}`;
		} catch (e) {
			previewError = e instanceof Error ? e.message : 'Failed to unhide port';
		} finally {
			previewSaving = false;
		}
	}
</script>

<div class="flex h-full flex-col">
	<!-- Header -->
	<div class="flex items-center gap-3 border-b border-border px-4 py-3">
		<button onclick={onBack} class="rounded-lg p-1 hover:bg-muted">
			<ArrowLeft class="h-5 w-5" />
		</button>
		<h2 class="flex-1 text-lg font-semibold">Settings</h2>
	</div>

	<!-- Settings content -->
	<div class="flex-1 overflow-y-auto p-4">
		<!-- Notifications -->
		<section class="mb-6">
			<h3 class="mb-3 text-sm font-medium text-muted-foreground">Notifications</h3>
			<div class="space-y-2">
				<div class="flex items-center justify-between rounded-xl bg-card p-4">
					<div class="flex items-center gap-3">
						<Bell class="h-5 w-5 text-muted-foreground" />
						<div>
							<div class="font-medium">Push Notifications</div>
							<div class="text-sm text-muted-foreground">
								{#if initializing}
									Loading state...
								{:else if subscribing}
									Updating subscription...
								{:else if pushError}
									{pushError}
								{:else if !pushSupported}
									Not supported in this browser
								{:else if pushPermission === 'denied'}
									Permission denied in browser settings
								{:else if notifications}
									Subscribed on this device
								{:else}
									Not subscribed
								{/if}
							</div>
							<div class="text-xs text-muted-foreground">
								Permission: {pushPermission}
								{#if pushEndpoint}
									· Synced endpoint
								{/if}
							</div>
						</div>
					</div>
					<button
						onclick={toggleNotifications}
						disabled={initializing || subscribing || !pushSupported}
						aria-label="Toggle push notifications"
						class="relative h-6 w-11 rounded-full transition-colors
							{notifications ? 'bg-primary' : 'bg-muted'}
							{initializing || subscribing || !pushSupported ? 'opacity-50' : ''}"
					>
						<span
							class="absolute top-0.5 h-5 w-5 rounded-full bg-white shadow transition-transform
								{notifications ? 'left-[22px]' : 'left-0.5'}"
						></span>
					</button>
				</div>

				<div class="rounded-xl bg-card p-4">
					<div class="mb-3 flex items-center gap-3">
						<Server class="h-5 w-5 text-muted-foreground" />
						<div>
							<div class="font-medium">Delivery Diagnostics</div>
							<div class="text-sm text-muted-foreground">
								Server health and recent push delivery outcomes
							</div>
						</div>
					</div>

					{#if statusError}
						<div class="mb-3 text-sm text-destructive">{statusError}</div>
					{:else if !pushStatus}
						<div class="mb-3 flex items-center gap-2 text-sm text-muted-foreground">
							<Loader2 class="h-4 w-4 animate-spin" />
							Loading push status...
						</div>
					{:else}
						<div class="mb-3 grid grid-cols-2 gap-x-3 gap-y-1 text-sm">
							<div class="text-muted-foreground">Configured</div>
							<div>{pushStatus.push_configured ? 'Yes' : 'No'}</div>
							<div class="text-muted-foreground">Subscriptions</div>
							<div>{pushStatus.subscription_count}</div>
							<div class="text-muted-foreground">Last Attempt</div>
							<div>{pushStatus.last_attempt_at ?? 'Never'}</div>
							<div class="text-muted-foreground">Last Success</div>
							<div>{pushStatus.last_success_at ?? 'Never'}</div>
							<div class="text-muted-foreground">Failures (24h)</div>
							<div>{pushStatus.recent_failures_24h}</div>
						</div>
						{#if pushStatus.last_failure_reason}
							<div class="mb-3 text-xs text-destructive">
								Last failure: {pushStatus.last_failure_reason}
							</div>
						{/if}
					{/if}

					<button
						onclick={sendTestNotification}
						disabled={testingPush || !notifications}
						class="flex w-full items-center justify-center gap-2 rounded-lg border border-input px-3 py-2 text-sm transition-colors hover:bg-muted disabled:opacity-50"
					>
						{#if testingPush}
							<Loader2 class="h-4 w-4 animate-spin" />
						{:else}
							<TestTube2 class="h-4 w-4" />
						{/if}
						Send Test Notification
					</button>
					{#if testMessage}
						<div class="mt-2 text-xs text-muted-foreground">{testMessage}</div>
					{/if}
				</div>
			</div>
			</section>

			<!-- Preview & Port Forwarding -->
			<section class="mb-6">
				<h3 class="mb-3 text-sm font-medium text-muted-foreground">Preview & Port Forwarding</h3>
				<div class="space-y-2">
					<div class="rounded-xl bg-card p-4">
						<div class="mb-3 flex items-center gap-3">
							<Monitor class="h-5 w-5 text-muted-foreground" />
							<div>
								<div class="font-medium">Preview Controls</div>
								<div class="text-sm text-muted-foreground">
									Manage auto-discovery and proxy policy for workspace previews
								</div>
							</div>
						</div>

						{#if previewLoading}
							<div class="mb-3 flex items-center gap-2 text-sm text-muted-foreground">
								<Loader2 class="h-4 w-4 animate-spin" />
								Loading preview settings...
							</div>
						{:else if !previewSettings}
							<div class="mb-3 text-sm text-destructive">Preview settings unavailable.</div>
						{:else}
							<div class="space-y-3">
								<div class="flex items-center justify-between rounded-lg border border-border p-3">
									<div>
										<div class="text-sm font-medium">Enable Preview Forwarding</div>
										<div class="text-xs text-muted-foreground">
											Use path-based forwarding through the current server port
										</div>
									</div>
									<button
										onclick={() => {
											if (previewSettings) previewSettings.enabled = !previewSettings.enabled;
										}}
										class="relative h-6 w-11 rounded-full transition-colors
											{previewSettings.enabled ? 'bg-primary' : 'bg-muted'}"
										aria-label="Toggle preview forwarding"
									>
										<span
											class="absolute top-0.5 h-5 w-5 rounded-full bg-white shadow transition-transform
												{previewSettings.enabled ? 'left-[22px]' : 'left-0.5'}"
										></span>
									</button>
								</div>

								<div class="grid grid-cols-1 gap-3 md:grid-cols-2">
									<label class="rounded-lg border border-border p-3">
										<div class="mb-1 text-sm font-medium">Auto Refresh (sec)</div>
										<input
											type="number"
											min="2"
											max="60"
											bind:value={previewSettings.auto_refresh_secs}
											class="w-full rounded-md border border-input bg-background px-2 py-1.5 text-sm"
										/>
									</label>
									<div class="rounded-lg border border-border p-3">
										<div class="mb-1 text-sm font-medium">HTTP-like Filter</div>
										<button
											onclick={() => {
												if (previewSettings) {
													previewSettings.show_only_http_like = !previewSettings.show_only_http_like;
												}
											}}
											class="rounded-md border border-input px-2 py-1.5 text-sm hover:bg-muted"
										>
											{previewSettings.show_only_http_like
												? 'Show HTTP-like only'
												: 'Show all discovered ports'}
										</button>
									</div>
								</div>

								<label class="block rounded-lg border border-border p-3">
									<div class="mb-1 text-sm font-medium">Blocked Ports</div>
									<div class="mb-2 text-xs text-muted-foreground">Comma-separated list</div>
									<input
										type="text"
										bind:value={blockedPortsInput}
										placeholder="22, 2375, 2376"
										class="w-full rounded-md border border-input bg-background px-2 py-1.5 text-sm"
									/>
								</label>

								<div class="grid gap-3 md:grid-cols-2">
									<div class="rounded-lg border border-border p-3">
										<div class="mb-2 flex items-center gap-2 text-sm font-medium">
											<Pin class="h-4 w-4" />
											Pinned Ports
										</div>
										<div class="mb-2 flex gap-2">
											<input
												type="number"
												min="1"
												max="65535"
												bind:value={newPinnedPort}
												placeholder="5173"
												class="w-full rounded-md border border-input bg-background px-2 py-1.5 text-sm"
											/>
											<button
												onclick={addPinnedPort}
												disabled={previewSaving}
												class="rounded-md border border-input px-3 py-1.5 text-sm hover:bg-muted disabled:opacity-50"
											>
												Add
											</button>
										</div>
										<div class="flex flex-wrap gap-2">
											{#if previewSettings.pinned_ports.length === 0}
												<span class="text-xs text-muted-foreground">No pinned ports</span>
											{:else}
												{#each previewSettings.pinned_ports as port}
													<span class="inline-flex items-center gap-1 rounded-full bg-muted px-2 py-0.5 text-xs">
														{port}
														<button
															onclick={() => removePinnedPort(port)}
															class="text-muted-foreground hover:text-foreground"
															aria-label={`Remove pinned port ${port}`}
														>
															<Trash2 class="h-3 w-3" />
														</button>
													</span>
												{/each}
											{/if}
										</div>
									</div>

									<div class="rounded-lg border border-border p-3">
										<div class="mb-2 flex items-center gap-2 text-sm font-medium">
											<EyeOff class="h-4 w-4" />
											Hidden Ports
										</div>
										<div class="mb-2 flex gap-2">
											<input
												type="number"
												min="1"
												max="65535"
												bind:value={newHiddenPort}
												placeholder="3001"
												class="w-full rounded-md border border-input bg-background px-2 py-1.5 text-sm"
											/>
											<button
												onclick={addHiddenPort}
												disabled={previewSaving}
												class="rounded-md border border-input px-3 py-1.5 text-sm hover:bg-muted disabled:opacity-50"
											>
												Add
											</button>
										</div>
										<div class="flex flex-wrap gap-2">
											{#if previewSettings.hidden_ports.length === 0}
												<span class="text-xs text-muted-foreground">No hidden ports</span>
											{:else}
												{#each previewSettings.hidden_ports as port}
													<span class="inline-flex items-center gap-1 rounded-full bg-muted px-2 py-0.5 text-xs">
														{port}
														<button
															onclick={() => removeHiddenPort(port)}
															class="text-muted-foreground hover:text-foreground"
															aria-label={`Remove hidden port ${port}`}
														>
															<Trash2 class="h-3 w-3" />
														</button>
													</span>
												{/each}
											{/if}
										</div>
									</div>
								</div>

								<div class="flex items-center justify-between gap-2">
									<div class="text-xs text-muted-foreground">
										Forwarding is served through the existing Krusty server port.
									</div>
									<button
										onclick={savePreviewSettings}
										disabled={previewSaving}
										class="inline-flex items-center gap-1 rounded-md border border-input px-3 py-1.5 text-sm hover:bg-muted disabled:opacity-50"
									>
										{#if previewSaving}
											<Loader2 class="h-4 w-4 animate-spin" />
										{:else}
											<Save class="h-4 w-4" />
										{/if}
										Save
									</button>
								</div>
							</div>
						{/if}

						{#if previewError}
							<div class="mt-3 text-sm text-destructive">{previewError}</div>
						{/if}
						{#if previewMessage}
							<div class="mt-2 text-xs text-muted-foreground">{previewMessage}</div>
						{/if}
					</div>
				</div>
			</section>

			<!-- Connection -->
			<section>
			<h3 class="mb-3 text-sm font-medium text-muted-foreground">Connection</h3>
			<div class="space-y-2">
				<div class="rounded-xl bg-card p-4">
					<div class="mb-3 flex items-center gap-3">
						<Server class="h-5 w-5 text-muted-foreground" />
						<div>
							<div class="font-medium">Server URL</div>
							<div class="text-sm text-muted-foreground">Krusty server address</div>
						</div>
					</div>
					<input
						type="url"
						bind:value={serverUrl}
						class="w-full rounded-lg border border-input bg-background px-3 py-2 text-sm
							placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
					/>
				</div>
			</div>
		</section>
	</div>
</div>
