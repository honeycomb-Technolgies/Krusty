<script lang="ts">
	import { ArrowLeft, Bell, Server } from 'lucide-svelte';
	import { subscribeToPush, unsubscribeFromPush, isPushSubscribed, isPushSupported } from '$lib/push';

	interface Props {
		onBack: () => void;
	}

	let { onBack }: Props = $props();

	let notifications = $state(isPushSubscribed());
	let subscribing = $state(false);
	let pushError = $state('');
	let serverUrl = $state('http://localhost:3000');

	async function toggleNotifications() {
		if (subscribing) return;
		subscribing = true;
		pushError = '';

		try {
			if (notifications) {
				await unsubscribeFromPush();
				notifications = false;
			} else {
				const ok = await subscribeToPush();
				if (ok) {
					notifications = true;
				} else {
					pushError = 'Permission denied';
				}
			}
		} catch (e) {
			pushError = e instanceof Error ? e.message : 'Failed';
		} finally {
			subscribing = false;
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
								{#if subscribing}
									Subscribing...
								{:else if pushError}
									{pushError}
								{:else if !isPushSupported()}
									Not supported in this browser
								{:else}
									Get notified on responses
								{/if}
							</div>
						</div>
					</div>
					<button
						onclick={toggleNotifications}
						disabled={subscribing || !isPushSupported()}
						aria-label="Toggle push notifications"
						class="relative h-6 w-11 rounded-full transition-colors
							{notifications ? 'bg-primary' : 'bg-muted'}
							{subscribing || !isPushSupported() ? 'opacity-50' : ''}"
					>
						<span
							class="absolute top-0.5 h-5 w-5 rounded-full bg-white shadow transition-transform
								{notifications ? 'left-[22px]' : 'left-0.5'}"
						></span>
					</button>
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
