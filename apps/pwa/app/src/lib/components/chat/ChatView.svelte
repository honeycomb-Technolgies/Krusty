<script lang="ts">
	import { onMount } from 'svelte';
	import Send from 'lucide-svelte/icons/send';
	import Loader2 from 'lucide-svelte/icons/loader-2';
	import StopCircle from 'lucide-svelte/icons/stop-circle';
	import Paperclip from 'lucide-svelte/icons/paperclip';
	import ImagePlus from 'lucide-svelte/icons/image-plus';
	import Message from './Message.svelte';
	import AsciiTitle from './AsciiTitle.svelte';
	import { sessionStore, sendMessage, stopGeneration, type Attachment } from '$stores/session';

	let inputValue = $state('');
	let inputElement = $state<HTMLTextAreaElement>(undefined!);
	let messagesContainer = $state<HTMLDivElement>(undefined!);
	let fileInput = $state<HTMLInputElement>(undefined!);
	let imageInput = $state<HTMLInputElement>(undefined!);
	let attachedFiles = $state<File[]>([]);

	function handleSubmit() {
		if (!inputValue.trim() || $sessionStore.isStreaming) return;

		// Convert files to attachments
		const attachments: Attachment[] = attachedFiles.map(file => ({
			file,
			type: file.type.startsWith('image/') ? 'image' : 'file'
		}));

		sendMessage(inputValue.trim(), attachments);
		inputValue = '';
		attachedFiles = [];
	}

	function handleFileSelect(e: Event) {
		const input = e.target as HTMLInputElement;
		if (input.files) {
			attachedFiles = [...attachedFiles, ...Array.from(input.files)];
		}
		input.value = '';
	}

	function removeFile(index: number) {
		attachedFiles = attachedFiles.filter((_, i) => i !== index);
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			handleSubmit();
		}
	}

	function autoResize() {
		if (inputElement) {
			inputElement.style.height = 'auto';
			inputElement.style.height = Math.min(inputElement.scrollHeight, 200) + 'px';
		}
	}

	$effect(() => {
		// Auto-scroll on new messages
		if ($sessionStore.messages.length && messagesContainer) {
			messagesContainer.scrollTop = messagesContainer.scrollHeight;
		}
	});

	onMount(() => {
		inputElement?.focus();
	});
</script>

<div class="flex h-full min-h-0 flex-col">
	<!-- Messages area -->
	<div
		bind:this={messagesContainer}
		class="messages-scroll min-h-0 flex-1 overflow-y-auto px-4 py-4"
	>
		{#if !$sessionStore.sessionId && $sessionStore.messages.length === 0}
			<!-- Welcome state - animated ASCII title -->
			<div class="flex h-full flex-col items-center justify-center">
				<AsciiTitle />
			</div>
		{:else if $sessionStore.messages.length === 0}
			<!-- Active session but no messages yet -->
		{:else}
			<div class="mx-auto max-w-3xl space-y-4">
				{#each $sessionStore.messages as message, i (i)}
					<Message {message} isStreaming={$sessionStore.isStreaming && i === $sessionStore.messages.length - 1} />
				{/each}
			</div>
		{/if}
	</div>

	<!-- Input area - only show when session is active -->
	{#if $sessionStore.sessionId}
		<div class="shrink-0 px-4 pb-5">
			<!-- Hidden file inputs -->
			<input
				bind:this={fileInput}
				type="file"
				multiple
				class="hidden"
				onchange={handleFileSelect}
			/>
			<input
				bind:this={imageInput}
				type="file"
				accept="image/*"
				multiple
				class="hidden"
				onchange={handleFileSelect}
			/>

			<!-- Attached files preview -->
			{#if attachedFiles.length > 0}
				<div class="mx-auto mb-2 flex max-w-3xl flex-wrap gap-2">
					{#each attachedFiles as file, i}
						<div class="flex items-center gap-1 rounded-lg bg-muted px-2 py-1 text-xs">
							<span class="max-w-[150px] truncate">{file.name}</span>
							<button
								onclick={() => removeFile(i)}
								class="ml-1 text-muted-foreground hover:text-foreground"
							>×</button>
						</div>
					{/each}
				</div>
			{/if}

			<div class="mx-auto max-w-3xl">
				<div class="flex items-end gap-2 rounded-xl border border-border/50 bg-card/60 backdrop-blur-sm p-2">
					<!-- Attachment buttons -->
					<button
						onclick={() => fileInput.click()}
						disabled={$sessionStore.isStreaming}
						class="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg text-muted-foreground
							transition-colors hover:bg-muted hover:text-foreground disabled:opacity-50"
						title="Attach file"
					>
						<Paperclip class="h-4 w-4" />
					</button>
					<button
						onclick={() => imageInput.click()}
						disabled={$sessionStore.isStreaming}
						class="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg text-muted-foreground
							transition-colors hover:bg-muted hover:text-foreground disabled:opacity-50"
						title="Attach image"
					>
						<ImagePlus class="h-4 w-4" />
					</button>

					<!-- Text input -->
					<textarea
						bind:this={inputElement}
						bind:value={inputValue}
						onkeydown={handleKeyDown}
						oninput={autoResize}
						placeholder="Message Krusty..."
						rows={1}
						disabled={$sessionStore.isStreaming}
						class="max-h-[200px] min-h-[36px] flex-1 resize-none bg-transparent py-2 text-sm
							placeholder:text-muted-foreground focus:outline-none disabled:cursor-not-allowed disabled:opacity-50"
					></textarea>

					<!-- Send/Stop button -->
					{#if $sessionStore.isStreaming}
						<button
							onclick={stopGeneration}
							class="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg
								bg-destructive text-destructive-foreground transition-colors hover:bg-destructive/90"
						>
							<StopCircle class="h-4 w-4" />
						</button>
					{:else}
						<button
							onclick={handleSubmit}
							disabled={!inputValue.trim()}
							class="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg
								bg-primary text-primary-foreground transition-colors
								hover:bg-primary/90 disabled:cursor-not-allowed disabled:opacity-50"
						>
							{#if $sessionStore.isLoading}
								<Loader2 class="h-4 w-4 animate-spin" />
							{:else}
								<Send class="h-4 w-4" />
							{/if}
						</button>
					{/if}
				</div>
			</div>
		</div>
	{/if}
</div>

<style>
	.messages-scroll {
		/* Prevent over-scroll bounce */
		overscroll-behavior: contain;

		/* GPU acceleration for smooth scrolling */
		will-change: scroll-position;
		transform: translateZ(0);

		/* Contain layout for performance */
		contain: strict;

		/* Smooth native scrolling on touch devices */
		-webkit-overflow-scrolling: touch;

		/* Hide scrollbar but keep functionality */
		scrollbar-width: thin;
		scrollbar-color: hsl(var(--muted-foreground) / 0.3) transparent;
	}

	.messages-scroll::-webkit-scrollbar {
		width: 6px;
	}

	.messages-scroll::-webkit-scrollbar-track {
		background: transparent;
	}

	.messages-scroll::-webkit-scrollbar-thumb {
		background: hsl(var(--muted-foreground) / 0.3);
		border-radius: 3px;
	}

	.messages-scroll::-webkit-scrollbar-thumb:hover {
		background: hsl(var(--muted-foreground) / 0.5);
	}
</style>
