<script lang="ts">
	import { onMount } from 'svelte';
	import Send from 'lucide-svelte/icons/send';
	import Clock from 'lucide-svelte/icons/clock';
	import StopCircle from 'lucide-svelte/icons/stop-circle';
	import Paperclip from 'lucide-svelte/icons/paperclip';
	import Shield from 'lucide-svelte/icons/shield';
	import Zap from 'lucide-svelte/icons/zap';
	import Message from './Message.svelte';
	import AsciiTitle from './AsciiTitle.svelte';
	import { sessionStore, sendMessage, stopGeneration, togglePermissionMode, type Attachment } from '$stores/session';

	let inputValue = $state('');
	let inputElement = $state<HTMLTextAreaElement>(undefined!);
	let messagesContainer = $state<HTMLDivElement>(undefined!);
	let fileInput = $state<HTMLInputElement>(undefined!);
	let attachedFiles = $state<File[]>([]);

	function handleSubmit() {
		if (!inputValue.trim()) return;

		// Convert files to attachments
		const attachments: Attachment[] = attachedFiles.map(file => ({
			file,
			type: file.type.startsWith('image/') ? 'image' : 'file'
		}));

		sendMessage(inputValue.trim(), attachments);
		inputValue = '';
		attachedFiles = [];
		// Reset input height after clearing
		autoResize();
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
			// If empty, clear height to use min-h from CSS/Tailwind
			if (!inputElement.value) {
				inputElement.style.height = '';
				return;
			}
			// Otherwise expand to content
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
			<!-- Hidden file input - accepts all file types including images -->
			<input
				bind:this={fileInput}
				type="file"
				accept="image/*,.pdf,.txt,.md,.json,.js,.ts,.html,.css,.py,.rs,.go,.java,.c,.cpp,.h,.sh,.yaml,.yml,.toml,.xml,.csv"
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
					<!-- Attachment button - accepts files and images -->
					<button
						onclick={() => fileInput.click()}
						class="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg text-muted-foreground
							transition-colors hover:bg-muted hover:text-foreground"
						title="Attach file or image"
					>
						<Paperclip class="h-4 w-4" />
					</button>

					<!-- Permission mode toggle -->
					<button
						onclick={togglePermissionMode}
						class="flex h-8 shrink-0 items-center gap-1 rounded-lg px-2 text-xs font-medium transition-colors
							{$sessionStore.permissionMode === 'supervised'
								? 'bg-amber-500/15 text-amber-400 hover:bg-amber-500/25'
								: 'bg-green-500/15 text-green-400 hover:bg-green-500/25'}"
						title="{$sessionStore.permissionMode === 'supervised' ? 'Supervised: approves write tools' : 'Autonomous: auto-executes all tools'}"
					>
						{#if $sessionStore.permissionMode === 'supervised'}
							<Shield class="h-3.5 w-3.5" />
						{:else}
							<Zap class="h-3.5 w-3.5" />
						{/if}
					</button>

					<!-- Text input -->
					<textarea
						bind:this={inputElement}
						bind:value={inputValue}
						onkeydown={handleKeyDown}
						oninput={autoResize}
						placeholder={$sessionStore.isStreaming ? 'Queue a message...' : 'Message Krusty...'}
						rows={1}
						inputmode="text"
						enterkeyhint="send"
						class="max-h-[200px] min-h-[36px] flex-1 resize-none bg-transparent py-2 text-sm
							placeholder:text-muted-foreground focus:outline-none"
					></textarea>

					<!-- Combined Send/Queue/Stop button -->
					<button
						onclick={$sessionStore.isStreaming ? (inputValue.trim() ? handleSubmit : stopGeneration) : handleSubmit}
						disabled={!$sessionStore.isStreaming && !inputValue.trim()}
						class="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg transition-colors
							{$sessionStore.isStreaming
								? inputValue.trim()
									? 'bg-amber-500 text-white hover:bg-amber-600'  // Queue
									: 'bg-destructive text-white hover:bg-destructive/90'  // Stop
								: 'bg-primary text-primary-foreground hover:bg-primary/90'  // Send (white text)
							}
							{!$sessionStore.isStreaming && !inputValue.trim() ? 'disabled:cursor-not-allowed disabled:opacity-50' : ''}"
						title={$sessionStore.isStreaming
							? (inputValue.trim() ? 'Queue message' : 'Stop generation')
							: 'Send message'
						}
					>
						{#if $sessionStore.isStreaming && inputValue.trim()}
							<Clock class="h-4 w-4" />
						{:else if $sessionStore.isStreaming}
							<StopCircle class="h-4 w-4" />
						{:else}
							<Send class="h-4 w-4" />
						{/if}
					</button>
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
