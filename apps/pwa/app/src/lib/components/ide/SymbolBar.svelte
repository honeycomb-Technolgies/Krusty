<script lang="ts">
	interface Props {
		onInsert: (symbol: string) => void;
	}

	let { onInsert }: Props = $props();

	// Common programming symbols organized in rows
	const symbolRows = [
		['(', ')', '{', '}', '[', ']', '<', '>', '/', '\\'],
		['=', '+', '-', '*', '%', '&', '|', '!', '?', ':'],
		[';', ',', '.', "'", '"', '`', '_', '#', '@', '$']
	];
</script>

<div class="symbol-bar safe-bottom">
	{#each symbolRows as row, i}
		<div class="symbol-row">
			{#each row as symbol}
				<button onclick={() => onInsert(symbol)} class="symbol-btn">
					{symbol}
				</button>
			{/each}
		</div>
	{/each}
</div>

<style>
	.symbol-bar {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		padding: 0.5rem;
		border-top: 1px solid hsl(var(--border) / 0.5);
		background: hsl(var(--card) / 0.95);
		backdrop-filter: blur(8px);
		-webkit-backdrop-filter: blur(8px);
	}

	.symbol-row {
		display: flex;
		gap: 0.25rem;
		justify-content: space-between;
	}

	.symbol-btn {
		flex: 1;
		min-width: 0;
		height: 2.25rem;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 0.375rem;
		font-size: 1rem;
		font-family: ui-monospace, SFMono-Regular, 'SF Mono', monospace;
		background: hsl(var(--muted));
		color: hsl(var(--foreground));
		border: 1px solid hsl(var(--border) / 0.5);
		transition: all 0.1s ease;
		touch-action: manipulation;
		-webkit-tap-highlight-color: transparent;
	}

	.symbol-btn:hover {
		background: hsl(var(--accent));
	}

	.symbol-btn:active {
		transform: scale(0.92);
		background: hsl(var(--primary) / 0.2);
	}

	/* Safe area for mobile home indicator */
	.safe-bottom {
		padding-bottom: max(0.5rem, env(safe-area-inset-bottom));
	}
</style>
