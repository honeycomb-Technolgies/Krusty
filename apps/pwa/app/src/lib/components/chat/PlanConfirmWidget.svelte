<script lang="ts">
	import FileText from 'lucide-svelte/icons/file-text';
	import Play from 'lucide-svelte/icons/play';
	import Trash2 from 'lucide-svelte/icons/trash-2';
	import Check from 'lucide-svelte/icons/check';
	import type { ToolCall } from '$stores/session';
	import { setMode, submitToolResult } from '$stores/session';

	interface Props {
		toolCall: ToolCall;
		planTitle?: string;
		taskCount?: number;
	}

	let { toolCall, planTitle = 'Implementation Plan', taskCount = 0 }: Props = $props();

	let isSubmitting = $state(false);
	let didSubmit = $state(false);
	const hasSubmitted = $derived(toolCall.status === 'success' || didSubmit);

	async function handleChoice(choice: 'execute' | 'abandon') {
		if (isSubmitting || hasSubmitted) return;

		isSubmitting = true;
		didSubmit = true;

		try {
			if (choice === 'execute') {
				// Switch to build mode before continuing execution
				setMode('build');
			}
			await submitToolResult(toolCall.id, JSON.stringify({ choice }));
		} catch (err) {
			console.error('[PlanConfirm] Failed to submit:', err);
			didSubmit = false;
		} finally {
			isSubmitting = false;
		}
	}
</script>

<div class="plan-confirm rounded-xl border border-green-500/30 bg-green-500/5 overflow-hidden">
	<!-- Header -->
	<div class="flex items-center gap-2 border-b border-green-500/20 bg-green-500/10 px-4 py-2.5">
		<FileText class="h-4 w-4 text-green-400" />
		<span class="text-sm font-medium text-green-400">Plan Ready</span>
		{#if hasSubmitted}
			<span class="ml-auto flex items-center gap-1 text-xs text-green-400">
				<Check class="h-3.5 w-3.5" />
				Confirmed
			</span>
		{/if}
	</div>

	<!-- Content -->
	<div class="p-4 space-y-3">
		<div class="text-sm text-foreground">
			<span class="font-medium">{planTitle}</span>
			{#if taskCount > 0}
				<span class="text-muted-foreground"> ({taskCount} tasks)</span>
			{/if}
		</div>

		<p class="text-sm text-muted-foreground">
			Ready to execute this plan? Choose Execute to switch to Build mode and start implementation, or Abandon to discard.
		</p>

		<!-- Buttons -->
		{#if !hasSubmitted}
			<div class="flex gap-2">
				<button
					onclick={() => handleChoice('execute')}
					disabled={isSubmitting}
					class="flex flex-1 items-center justify-center gap-2 rounded-lg bg-green-500 px-4 py-2.5 text-sm font-medium text-white
						transition-colors hover:bg-green-600 disabled:cursor-not-allowed disabled:opacity-50"
				>
					{#if isSubmitting}
						<div class="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent"></div>
					{:else}
						<Play class="h-4 w-4" />
					{/if}
					Execute
				</button>
				<button
					onclick={() => handleChoice('abandon')}
					disabled={isSubmitting}
					class="flex flex-1 items-center justify-center gap-2 rounded-lg border border-border bg-muted/50 px-4 py-2.5 text-sm font-medium
						transition-colors hover:bg-muted disabled:cursor-not-allowed disabled:opacity-50"
				>
					<Trash2 class="h-4 w-4" />
					Abandon
				</button>
			</div>
		{/if}
	</div>
</div>

<style>
	.plan-confirm {
		contain: layout style paint;
	}
</style>
