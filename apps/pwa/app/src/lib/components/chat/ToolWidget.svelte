<script lang="ts">
	import Terminal from 'lucide-svelte/icons/terminal';
	import FileText from 'lucide-svelte/icons/file-text';
	import FolderTree from 'lucide-svelte/icons/folder-tree';
	import Search from 'lucide-svelte/icons/search';
	import Edit3 from 'lucide-svelte/icons/edit-3';
	import CheckIcon from 'lucide-svelte/icons/check';
	import X from 'lucide-svelte/icons/x';
	import ChevronDown from 'lucide-svelte/icons/chevron-down';
	import ChevronUp from 'lucide-svelte/icons/chevron-up';
	import Loader2 from 'lucide-svelte/icons/loader-2';
	import type { ToolCall } from '$stores/session';

	interface Props {
		toolCall: ToolCall;
	}

	let { toolCall }: Props = $props();
	let isExpanded = $state(false);
	let elapsedSeconds = $state(0);
	let startTime: number | null = $state(null);
	let timerInterval: ReturnType<typeof setInterval> | null = null;

	// Track elapsed time while tool is running
	$effect(() => {
		if (toolCall.status === 'running') {
			if (!startTime) startTime = Date.now();
			timerInterval = setInterval(() => {
				elapsedSeconds = Math.floor((Date.now() - startTime!) / 1000);
			}, 1000);
		} else {
			if (timerInterval) {
				clearInterval(timerInterval);
				timerInterval = null;
			}
		}

		return () => {
			if (timerInterval) clearInterval(timerInterval);
		};
	});

	// Auto-expand bash tools when streaming output starts
	$effect(() => {
		if (toolCall.name === 'bash' && toolCall.status === 'running' && toolCall.output) {
			isExpanded = true;
		}
	});

	const iconMap: Record<string, typeof Terminal> = {
		bash: Terminal,
		read: FileText,
		write: Edit3,
		glob: FolderTree,
		grep: Search
	};

	const Icon = $derived(iconMap[toolCall.name] || Terminal);

	const statusStyles = $derived({
		running: 'border-blue-500/50 bg-blue-500/10',
		success: 'border-green-500/50 bg-green-500/10',
		error: 'border-red-500/50 bg-red-500/10',
		pending: 'border-border bg-muted/50'
	}[toolCall.status] || 'border-border bg-muted/50');

	// Format tool description from arguments
	const description = $derived(() => {
		if (toolCall.description) return toolCall.description;
		if (!toolCall.arguments) return null;

		const args = toolCall.arguments;
		switch (toolCall.name) {
			case 'bash':
				return args.command as string;
			case 'read':
				return args.file_path as string;
			case 'write':
				return args.file_path as string;
			case 'glob':
				return args.pattern as string;
			case 'grep':
				return `${args.pattern} in ${args.path || '.'}`;
			default:
				return null;
		}
	});
</script>

<div class="tool-widget rounded-xl border {statusStyles} overflow-hidden">
	<button
		onclick={() => (isExpanded = !isExpanded)}
		class="flex w-full items-center gap-3 px-4 py-2.5 text-left text-sm"
	>
		<div class="flex h-6 w-6 shrink-0 items-center justify-center rounded-md bg-background/50">
			<Icon class="h-3.5 w-3.5" />
		</div>

		<div class="flex-1 min-w-0">
			<div class="flex items-center gap-2">
				<span class="font-medium">{toolCall.name}</span>
				{#if toolCall.status === 'running'}
					<Loader2 class="h-3.5 w-3.5 animate-spin text-blue-500" />
					<span class="text-xs text-muted-foreground tabular-nums">{elapsedSeconds}s</span>
				{:else if toolCall.status === 'success'}
					<CheckIcon class="h-3.5 w-3.5 text-green-500" />
				{:else if toolCall.status === 'error'}
					<X class="h-3.5 w-3.5 text-red-500" />
				{/if}
			</div>
			{#if description()}
				<div class="truncate text-xs text-muted-foreground font-mono">{description()}</div>
			{/if}
		</div>

		<div class="shrink-0">
			{#if isExpanded}
				<ChevronUp class="h-4 w-4 text-muted-foreground" />
			{:else}
				<ChevronDown class="h-4 w-4 text-muted-foreground" />
			{/if}
		</div>
	</button>

	{#if isExpanded}
		<div class="border-t border-border/50 bg-background/30 p-3 space-y-3">
			{#if toolCall.arguments}
				<div>
					<div class="mb-1 text-xs font-medium text-muted-foreground">Input</div>
					<pre class="rounded-lg bg-muted/50 p-2.5 font-mono text-xs overflow-x-auto">{JSON.stringify(toolCall.arguments, null, 2)}</pre>
				</div>
			{/if}

			{#if toolCall.output}
				<div>
					<div class="mb-1 text-xs font-medium text-muted-foreground">Output</div>
					<pre class="rounded-lg bg-muted/50 p-2.5 font-mono text-xs overflow-x-auto max-h-48 overflow-y-auto whitespace-pre-wrap">{toolCall.output}</pre>
				</div>
			{/if}
		</div>
	{/if}
</div>

<style>
	.tool-widget {
		contain: layout style paint;
	}
</style>
