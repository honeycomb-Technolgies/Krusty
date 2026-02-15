<script lang="ts">
	import HelpCircle from 'lucide-svelte/icons/help-circle';
	import Check from 'lucide-svelte/icons/check';
	import Send from 'lucide-svelte/icons/send';
	import type { ToolCall } from '$stores/session';
	import { submitToolResult } from '$stores/session';

	interface Props {
		toolCall: ToolCall;
	}

	let { toolCall }: Props = $props();

	interface Question {
		question: string;
		header: string;
		options: Array<{ label: string; description?: string }>;
		multiSelect?: boolean;
	}

	// Parse the tool arguments
	const questions = $derived<Question[]>(
		(toolCall.arguments?.questions as Question[]) || []
	);

	// Track selected options per question
	let selections = $state<Record<number, Set<string>>>({});
	let customInputs = $state<Record<number, string>>({});
	let isSubmitting = $state(false);
	let didSubmit = $state(false);
	const hasSubmitted = $derived(toolCall.status === 'success' || didSubmit);

	function toggleOption(questionIdx: number, label: string, multiSelect: boolean) {
		if (hasSubmitted) return;

		if (!selections[questionIdx]) {
			selections[questionIdx] = new Set();
		}

		if (multiSelect) {
			if (selections[questionIdx].has(label)) {
				selections[questionIdx].delete(label);
			} else {
				selections[questionIdx].add(label);
			}
		} else {
			// Single select - clear and set
			selections[questionIdx] = new Set([label]);
		}
		// Trigger reactivity
		selections = { ...selections };
	}

	function isSelected(questionIdx: number, label: string): boolean {
		return selections[questionIdx]?.has(label) ?? false;
	}

	async function handleSubmit() {
		console.log('[AskUserQuestion] handleSubmit called', { isSubmitting, hasSubmitted });
		if (isSubmitting || hasSubmitted) return;

		const answers: Record<string, string | string[]> = {};
		const qs = questions;

		for (let i = 0; i < qs.length; i++) {
			const q = qs[i];
			const selected = Array.from(selections[i] || []);
			const custom = customInputs[i]?.trim();

			if (custom) {
				answers[q.header] = custom;
			} else if (selected.length > 0) {
				// Multi-select returns array, single select returns string
				if (q.multiSelect) {
					answers[q.header] = selected;
				} else {
					answers[q.header] = selected[0];
				}
			}
		}

		// Format as {"answers": {...}} to match TUI format
		const result = JSON.stringify({ answers });
		console.log('[AskUserQuestion] Submitting:', { toolCallId: toolCall.id, result });

		isSubmitting = true;
		try {
			await submitToolResult(toolCall.id, result);
			console.log('[AskUserQuestion] Submit completed');
			didSubmit = true;
		} catch (err) {
			console.error('[AskUserQuestion] Failed to submit:', err);
		} finally {
			isSubmitting = false;
		}
	}

	const canSubmit = $derived.by(() => {
		const qs = questions;
		if (qs.length === 0) return false;

		for (let i = 0; i < qs.length; i++) {
			const hasSelection = (selections[i]?.size ?? 0) > 0;
			const hasCustom = !!customInputs[i]?.trim();
			if (!hasSelection && !hasCustom) return false;
		}
		return true;
	});
</script>

<div class="ask-question rounded-xl border border-amber-500/30 bg-amber-500/5 overflow-hidden">
	<!-- Header -->
	<div class="flex items-center gap-2 border-b border-amber-500/20 bg-amber-500/10 px-4 py-2.5">
		<HelpCircle class="h-4 w-4 text-amber-400" />
		<span class="text-sm font-medium text-amber-400">Question</span>
		{#if hasSubmitted}
			<span class="ml-auto flex items-center gap-1 text-xs text-green-400">
				<Check class="h-3.5 w-3.5" />
				Answered
			</span>
		{/if}
	</div>

	<!-- Questions -->
	<div class="p-4 space-y-4">
		{#each questions as question, qIdx}
			<div class="space-y-2">
				<div class="flex items-center gap-2">
					<span class="rounded-full bg-amber-500/20 px-2 py-0.5 text-xs font-medium text-amber-400">
						{question.header}
					</span>
				</div>
				<p class="text-sm text-foreground">{question.question}</p>

				<!-- Options -->
				<div class="space-y-1.5">
					{#each question.options as option}
						<button
							onclick={() => toggleOption(qIdx, option.label, question.multiSelect ?? false)}
							disabled={hasSubmitted}
							class="flex w-full items-start gap-3 rounded-lg border px-3 py-2.5 text-left transition-colors
								{isSelected(qIdx, option.label)
									? 'border-amber-500 bg-amber-500/15 text-foreground'
									: 'border-border/50 bg-muted/30 text-muted-foreground hover:border-amber-500/50 hover:bg-muted/50'}
								{hasSubmitted ? 'cursor-default opacity-70' : 'cursor-pointer'}"
						>
							<div class="mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center rounded
								{question.multiSelect ? 'rounded-sm' : 'rounded-full'}
								border transition-colors
								{isSelected(qIdx, option.label)
									? 'border-amber-500 bg-amber-500 text-white'
									: 'border-muted-foreground/50'}"
							>
								{#if isSelected(qIdx, option.label)}
									<Check class="h-3 w-3" />
								{/if}
							</div>
							<div class="flex-1 min-w-0">
								<div class="text-sm font-medium">{option.label}</div>
								{#if option.description}
									<div class="text-xs text-muted-foreground mt-0.5">{option.description}</div>
								{/if}
							</div>
						</button>
					{/each}

					<!-- Custom input (Other) -->
					{#if !hasSubmitted}
						<div class="flex items-center gap-2 mt-2">
							<input
								type="text"
								bind:value={customInputs[qIdx]}
								oninput={() => {
									// Clear bubble selection when typing custom answer
									if (customInputs[qIdx]?.trim()) {
										selections[qIdx] = new Set();
										selections = { ...selections };
									}
								}}
								placeholder="Other (type your answer)..."
								class="flex-1 rounded-lg border border-border/50 bg-muted/30 px-3 py-2 text-sm
									placeholder:text-muted-foreground focus:border-amber-500/50 focus:outline-none focus:ring-1 focus:ring-amber-500/30"
							/>
						</div>
					{/if}
				</div>
			</div>
		{/each}

		<!-- Submit button -->
		{#if !hasSubmitted}
			<button
				onclick={handleSubmit}
				disabled={!canSubmit || isSubmitting}
				class="flex w-full items-center justify-center gap-2 rounded-lg bg-amber-500 px-4 py-2.5 text-sm font-medium text-white
					transition-colors hover:bg-amber-600 disabled:cursor-not-allowed disabled:opacity-50"
			>
				{#if isSubmitting}
					<div class="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent"></div>
					Submitting...
				{:else}
					<Send class="h-4 w-4" />
					Submit Answer
				{/if}
			</button>
		{/if}
	</div>
</div>

<style>
	.ask-question {
		contain: layout style paint;
	}
</style>
