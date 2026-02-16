<script lang="ts">
	import Shield from 'lucide-svelte/icons/shield';
	import Check from 'lucide-svelte/icons/check';
	import X from 'lucide-svelte/icons/x';
	import type { ToolCall } from '$stores/session';
	import { approveToolCall, denyToolCall } from '$stores/session';

	interface Props {
		toolCall: ToolCall;
	}

	let { toolCall }: Props = $props();
	let isSubmitting = $state(false);

	const isAwaiting = $derived(toolCall.status === 'awaiting_approval');

	async function handleApprove() {
		if (isSubmitting) return;
		isSubmitting = true;
		try {
			await approveToolCall(toolCall.id);
		} catch (err) {
			console.error('Failed to approve tool:', err);
		} finally {
			isSubmitting = false;
		}
	}

	async function handleDeny() {
		if (isSubmitting) return;
		isSubmitting = true;
		try {
			await denyToolCall(toolCall.id);
		} catch (err) {
			console.error('Failed to deny tool:', err);
		} finally {
			isSubmitting = false;
		}
	}
</script>

<div class="tool-approval rounded-xl border border-amber-500/30 bg-amber-500/5 overflow-hidden">
	<!-- Header -->
	<div class="flex items-center gap-2 border-b border-amber-500/20 bg-amber-500/10 px-4 py-2.5">
		<Shield class="h-4 w-4 text-amber-400" />
		<span class="text-sm font-medium text-amber-400">Permission Required: {toolCall.name}</span>
		{#if !isAwaiting}
			<span class="ml-auto flex items-center gap-1 text-xs {toolCall.status === 'error' ? 'text-red-400' : 'text-green-400'}">
				{#if toolCall.status === 'error'}
					<X class="h-3.5 w-3.5" />
					Denied
				{:else}
					<Check class="h-3.5 w-3.5" />
					Approved
				{/if}
			</span>
		{/if}
	</div>

	<!-- Arguments preview -->
	<div class="p-4 space-y-3">
		{#if toolCall.arguments}
			<div class="max-h-48 overflow-auto rounded-lg bg-background/50 border border-border/30 p-3">
				<pre class="text-xs text-muted-foreground whitespace-pre-wrap break-all">{JSON.stringify(toolCall.arguments, null, 2)}</pre>
			</div>
		{/if}

		<!-- Action buttons -->
		{#if isAwaiting}
			<div class="flex gap-2">
				<button
					onclick={handleApprove}
					disabled={isSubmitting}
					class="flex flex-1 items-center justify-center gap-2 rounded-lg bg-green-600 px-4 py-2.5 text-sm font-medium text-white
						transition-colors hover:bg-green-700 disabled:cursor-not-allowed disabled:opacity-50"
				>
					{#if isSubmitting}
						<div class="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent"></div>
					{:else}
						<Check class="h-4 w-4" />
					{/if}
					Approve
				</button>
				<button
					onclick={handleDeny}
					disabled={isSubmitting}
					class="flex flex-1 items-center justify-center gap-2 rounded-lg bg-muted px-4 py-2.5 text-sm font-medium text-muted-foreground
						transition-colors hover:bg-muted/80 disabled:cursor-not-allowed disabled:opacity-50"
				>
					<X class="h-4 w-4" />
					Deny
				</button>
			</div>
		{/if}
	</div>
</div>

<style>
	.tool-approval {
		contain: layout style paint;
	}
</style>
