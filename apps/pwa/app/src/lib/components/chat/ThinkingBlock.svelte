<script lang="ts">
	import Brain from 'lucide-svelte/icons/brain';
	import ChevronDown from 'lucide-svelte/icons/chevron-down';
	import ChevronUp from 'lucide-svelte/icons/chevron-up';

	interface Props {
		content: string;
		isStreaming?: boolean;
	}

	let { content, isStreaming = false }: Props = $props();
	let isExpanded = $state(false);
</script>

<div class="w-full">
	<button
		onclick={() => (isExpanded = !isExpanded)}
		class="flex w-full items-center gap-2 rounded-xl border border-purple-500/30 bg-purple-500/10
			px-3 py-2 text-left text-sm text-purple-300 transition-colors
			hover:border-purple-500/50 hover:bg-purple-500/15"
	>
		<Brain class="h-4 w-4 shrink-0" />
		{#if isStreaming}
			<div class="thinking-dots flex gap-1">
				<span></span>
				<span></span>
				<span></span>
			</div>
			<span class="flex-1 text-xs font-medium">Thinking...</span>
		{:else}
			<span class="flex-1 text-xs font-medium">Thought process</span>
		{/if}
		{#if isExpanded}
			<ChevronUp class="h-3.5 w-3.5" />
		{:else}
			<ChevronDown class="h-3.5 w-3.5" />
		{/if}
	</button>

	{#if isExpanded && content}
		<div class="mt-2 rounded-xl border border-purple-500/20 bg-purple-500/5 p-3">
			<pre class="whitespace-pre-wrap font-mono text-xs text-muted-foreground leading-relaxed">{content}</pre>
		</div>
	{/if}
</div>

<style>
	.thinking-dots span {
		width: 5px;
		height: 5px;
		background: currentColor;
		border-radius: 50%;
		animation: thinking 1.4s ease-in-out infinite;
	}

	.thinking-dots span:nth-child(1) { animation-delay: 0s; }
	.thinking-dots span:nth-child(2) { animation-delay: 0.2s; }
	.thinking-dots span:nth-child(3) { animation-delay: 0.4s; }

	@keyframes thinking {
		0%, 80%, 100% {
			opacity: 0.3;
			transform: scale(0.8);
		}
		40% {
			opacity: 1;
			transform: scale(1);
		}
	}
</style>
