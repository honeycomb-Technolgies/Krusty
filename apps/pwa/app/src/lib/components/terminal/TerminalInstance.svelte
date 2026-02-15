<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { browser } from '$app/environment';
	import { terminalStore, connectTerminal, disconnectTerminal, sendInput, sendResize } from '$stores/terminal';

	interface Props {
		tabId: string;
		isActive: boolean;
	}

	let { tabId, isActive }: Props = $props();

	let terminalContainer: HTMLDivElement;
	let terminal: any;
	let fitAddon: any;
	let resizeObserver: ResizeObserver | null = null;
	let initialized = false;

	// Get this tab's state
	let tabState = $derived($terminalStore.tabs.find((t) => t.id === tabId));

	onMount(() => {
		if (!browser) return;
		initTerminal();
	});

	async function initTerminal() {
		const { Terminal } = await import('@xterm/xterm');
		const { FitAddon } = await import('@xterm/addon-fit');
		const { WebLinksAddon } = await import('@xterm/addon-web-links');

		await import('@xterm/xterm/css/xterm.css');

		terminal = new Terminal({
			cursorBlink: true,
			cursorStyle: 'block',
			fontSize: 14,
			fontFamily: 'JetBrains Mono, Fira Code, monospace',
			theme: {
				background: '#0a0a0a',
				foreground: '#fafafa',
				cursor: '#fafafa',
				cursorAccent: '#0a0a0a',
				selectionBackground: '#3f3f46',
				black: '#18181b',
				red: '#ef4444',
				green: '#22c55e',
				yellow: '#eab308',
				blue: '#3b82f6',
				magenta: '#a855f7',
				cyan: '#06b6d4',
				white: '#fafafa',
				brightBlack: '#52525b',
				brightRed: '#f87171',
				brightGreen: '#4ade80',
				brightYellow: '#facc15',
				brightBlue: '#60a5fa',
				brightMagenta: '#c084fc',
				brightCyan: '#22d3ee',
				brightWhite: '#ffffff'
			}
		});

		fitAddon = new FitAddon();
		terminal.loadAddon(fitAddon);
		terminal.loadAddon(new WebLinksAddon());

		terminal.open(terminalContainer);

		function fitAndResize() {
			if (!terminal || !fitAddon) return;
			fitAddon.fit();
			sendResize(tabId, terminal.cols, terminal.rows);
		}

		requestAnimationFrame(() => fitAndResize());

		terminal.onData((data: string) => {
			sendInput(tabId, data);
		});

		let resizeTimeout: ReturnType<typeof setTimeout>;
		resizeObserver = new ResizeObserver(() => {
			clearTimeout(resizeTimeout);
			resizeTimeout = setTimeout(() => fitAndResize(), 50);
		});
		resizeObserver.observe(terminalContainer);

		connectTerminal(tabId, (data: string) => {
			terminal.write(data);
		});

		const unsubscribe = terminalStore.subscribe((state) => {
			const tab = state.tabs.find((t) => t.id === tabId);
			if (tab?.connected && terminal && !initialized) {
				initialized = true;
				setTimeout(() => fitAndResize(), 100);
				unsubscribe();
			}
		});
	}

	// Re-fit when becoming active
	$effect(() => {
		if (isActive && terminal && fitAddon) {
			requestAnimationFrame(() => {
				fitAddon.fit();
				sendResize(tabId, terminal.cols, terminal.rows);
			});
		}
	});

	onDestroy(() => {
		resizeObserver?.disconnect();
		disconnectTerminal(tabId);
		terminal?.dispose();
	});

	export function focus() {
		terminal?.focus();
	}

	export function handleInput(data: string) {
		sendInput(tabId, data);
	}
</script>

<div
	class="terminal-instance"
	class:hidden={!isActive}
>
	<div bind:this={terminalContainer} class="terminal-container"></div>

	{#if tabState && !tabState.connected}
		<div class="connection-overlay">
			<div class="connection-status">
				{#if tabState.error}
					<div class="error-icon">!</div>
					<p class="error-text">{tabState.error}</p>
					<button
						onclick={() => connectTerminal(tabId, (data) => terminal?.write(data))}
						class="reconnect-btn"
					>
						Reconnect
					</button>
				{:else}
					<div class="connecting-spinner"></div>
					<p class="connecting-text">Connecting...</p>
				{/if}
			</div>
		</div>
	{/if}
</div>

<style>
	.terminal-instance {
		position: relative;
		height: 100%;
		background: #0a0a0a;
	}

	.terminal-instance.hidden {
		display: none;
	}

	.terminal-container {
		height: 100%;
		padding: 0.5rem;
	}

	.terminal-container :global(.xterm) {
		height: 100%;
	}

	.connection-overlay {
		position: absolute;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		background: hsl(var(--background) / 0.9);
		backdrop-filter: blur(4px);
	}

	.connection-status {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 1rem;
		padding: 2rem;
		border-radius: 1rem;
		background: hsl(var(--card));
		border: 1px solid hsl(var(--border) / 0.5);
	}

	.error-icon {
		width: 3rem;
		height: 3rem;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 50%;
		background: hsl(var(--destructive) / 0.2);
		color: hsl(var(--destructive));
		font-size: 1.5rem;
		font-weight: bold;
	}

	.error-text {
		color: hsl(var(--destructive));
		font-size: 0.875rem;
	}

	.reconnect-btn {
		padding: 0.625rem 1.25rem;
		border-radius: 0.5rem;
		font-size: 0.875rem;
		font-weight: 500;
		background: linear-gradient(135deg, #ff6b35, #e85d04);
		color: white;
		border: none;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.reconnect-btn:hover {
		transform: translateY(-1px);
	}

	.connecting-spinner {
		width: 2rem;
		height: 2rem;
		border: 3px solid hsl(var(--muted));
		border-top-color: #ff6b35;
		border-radius: 50%;
		animation: spin 1s linear infinite;
	}

	.connecting-text {
		color: hsl(var(--muted-foreground));
		font-size: 0.875rem;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}
</style>
