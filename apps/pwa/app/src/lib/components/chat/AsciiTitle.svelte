<script lang="ts">
	// ASCII art for "KRUSTY" using block characters
	const lines = [
		'▄ •▄ ▄▄▄  ▄• ▄▌.▄▄ · ▄▄▄▄▄ ▄· ▄▌',
		'█▌▄▌▪▀▄ █·█▪██▌▐█ ▀. •██  ▐█▪██▌',
		'▐▀▀▄·▐▀▀▄ █▌▐█▌▄▀▀▀█▄ ▐█.▪▐█▌▐█▪',
		'▐█.█▌▐█•█▌▐█▄█▌▐█▄▪▐█ ▐█▌· ▐█▀·.',
		'·▀  ▀.▀  ▀ ▀▀▀  ▀▀▀▀  ▀▀▀   ▀ • '
	];
</script>

<div class="ascii-title select-none" aria-label="Krusty">
	{#each lines as line, i}
		<div
			class="line"
			style="--line-index: {i}"
		>{line}</div>
	{/each}
</div>

<style>
	.ascii-title {
		font-family: 'JetBrains Mono', monospace;
		font-size: clamp(0.5rem, 2vw, 1rem);
		line-height: 1.1;
		display: flex;
		flex-direction: column;
		align-items: center;
		letter-spacing: 0;
		font-variant-ligatures: none;
	}

	.line {
		white-space: pre;
		background: linear-gradient(
			90deg,
			#8b4513 0%,
			#cd853f 15%,
			#ff6b35 35%,
			#ffcc00 50%,
			#ff6b35 65%,
			#cd853f 85%,
			#8b4513 100%
		);
		background-size: 200% 100%;
		-webkit-background-clip: text;
		background-clip: text;
		-webkit-text-fill-color: transparent;
		animation:
			shimmer 4s ease-in-out infinite,
			wave 3s ease-in-out infinite;
		animation-delay:
			calc(var(--line-index) * 0.15s),
			calc(var(--line-index) * 0.1s);
	}

	@keyframes shimmer {
		0%, 100% {
			background-position: 100% 0;
		}
		50% {
			background-position: 0% 0;
		}
	}

	@keyframes wave {
		0%, 100% {
			transform: translateY(0) scaleY(1);
		}
		25% {
			transform: translateY(-2px) scaleY(1.02);
		}
		50% {
			transform: translateY(0) scaleY(1);
		}
		75% {
			transform: translateY(2px) scaleY(0.98);
		}
	}
</style>
