<script lang="ts">
	import type { KeyConfig, KeyboardLayout } from './layouts';
	import { getLayout, getTerminalSequence, formatKeyDisplay, KeyAction } from './layouts';

	interface Props {
		mode?: 'chat' | 'terminal';
		visible?: boolean;
		onKeyPress?: (key: string, isEnter: boolean) => void;
		onClose?: () => void;
		onHeightChange?: (height: number) => void;
	}

	let {
		mode = 'chat',
		visible = true,
		onKeyPress,
		onClose,
		onHeightChange
	}: Props = $props();

	let isTerminalMode = $derived(mode === 'terminal');

	// State — always QWERTY by default regardless of mode
	let currentLayout = $state<KeyboardLayout>('qwerty');
	let isShifted = $state(false);
	let keyboardEl: HTMLDivElement | undefined = $state(undefined);

	let layout = $derived(getLayout(currentLayout));

	// Report keyboard height to parent when visible
	$effect(() => {
		if (visible && keyboardEl) {
			const height = keyboardEl.offsetHeight;
			onHeightChange?.(height);
		} else if (!visible) {
			onHeightChange?.(0);
		}
	});

	// Key repeat state
	let repeatInterval: ReturnType<typeof setInterval> | null = null;
	const REPEAT_INTERVAL_MS = 100;

	function startRepeat(key: KeyConfig) {
		handleKeyAction(key);
		repeatInterval = setInterval(() => {
			handleKeyAction(key);
		}, REPEAT_INTERVAL_MS);
	}

	function stopRepeat() {
		if (repeatInterval) {
			clearInterval(repeatInterval);
			repeatInterval = null;
		}
	}

	// Key preview popup state
	let previewKey: KeyConfig | null = $state(null);
	let previewPosition = $state({ x: 0, y: 0 });
	let previewTimeout: ReturnType<typeof setTimeout> | null = null;

	function showPreview(key: KeyConfig, event: MouseEvent | TouchEvent) {
		if (key.action) return;
		const clientX = 'touches' in event ? event.touches[0].clientX : (event as MouseEvent).clientX;
		const clientY = 'touches' in event ? event.touches[0].clientY : (event as MouseEvent).clientY;
		previewPosition = { x: clientX, y: clientY - 60 };
		previewKey = key;
		if (previewTimeout) clearTimeout(previewTimeout);
	}

	function hidePreview() {
		if (previewTimeout) {
			clearTimeout(previewTimeout);
			previewTimeout = null;
		}
		previewKey = null;
	}

	function handleKeyAction(key: KeyConfig) {
		if (navigator.vibrate) navigator.vibrate(10);

		if (isTerminalMode) {
			handleTerminalKeyPress(key);
			return;
		}
		handleChatKeyPress(key);
	}

	function handleKeyPress(key: KeyConfig) {
		handleKeyAction(key);
	}

	function handleTerminalKeyPress(key: KeyConfig) {
		const action = key.action;
		const value = key.value;

		switch (action) {
			case KeyAction.Shift:
				isShifted = !isShifted;
				return;
			case KeyAction.Backspace:
				onKeyPress?.('\x7f', false);
				return;
			case KeyAction.Space:
				onKeyPress?.(' ', false);
				return;
			case KeyAction.Enter:
				onKeyPress?.('\n', true);
				return;
			case KeyAction.SwitchNumbers:
				currentLayout = 'numbers';
				return;
			case KeyAction.SwitchSymbols:
				currentLayout = 'symbols';
				return;
			case KeyAction.SwitchQwerty:
				currentLayout = 'qwerty';
				return;
		}

		// Try escape sequence for navigation keys, otherwise emit raw value
		const sequence = getTerminalSequence(value);
		if (sequence !== value) {
			onKeyPress?.(sequence, false);
		} else {
			let char = value;
			if (isShifted && char.length === 1 && char.match(/[a-z]/)) {
				char = char.toUpperCase();
			}
			onKeyPress?.(char, false);
			if (isShifted && currentLayout === 'qwerty') {
				isShifted = false;
			}
		}
	}

	function handleChatKeyPress(key: KeyConfig) {
		const action = key.action;

		switch (action) {
			case KeyAction.Shift:
				isShifted = !isShifted;
				break;
			case KeyAction.Backspace:
				onKeyPress?.('\x7f', false);
				break;
			case KeyAction.Space:
				onKeyPress?.(' ', false);
				break;
			case KeyAction.Enter:
				onKeyPress?.('\n', true);
				break;
			case KeyAction.SwitchNumbers:
				currentLayout = 'numbers';
				isShifted = false;
				break;
			case KeyAction.SwitchSymbols:
				currentLayout = 'symbols';
				isShifted = false;
				break;
			case KeyAction.SwitchQwerty:
				currentLayout = 'qwerty';
				isShifted = false;
				break;
			default: {
				let char = key.value;
				if (isShifted && char.length === 1 && char.match(/[a-z]/)) {
					char = char.toUpperCase();
				}
				onKeyPress?.(char, false);
				if (isShifted && currentLayout === 'qwerty') {
					isShifted = false;
				}
				break;
			}
		}
	}

	// Swipe down dismissal
	let touchStartY = $state(0);
	let touchStartTime = $state(0);
	const SWIPE_THRESHOLD = 100;
	const SWIPE_TIME_LIMIT = 500;

	function handleTouchStart(e: TouchEvent) {
		touchStartY = e.touches[0].clientY;
		touchStartTime = Date.now();
	}

	function handleTouchEnd(e: TouchEvent) {
		const deltaY = e.changedTouches[0].clientY - touchStartY;
		const elapsed = Date.now() - touchStartTime;
		if (deltaY > SWIPE_THRESHOLD && elapsed < SWIPE_TIME_LIMIT) {
			onClose?.();
		}
	}

	function handleBackdropClick(e: MouseEvent) {
		const target = e.target as HTMLElement;
		if (target.classList.contains('keyboard-backdrop')) {
			onClose?.();
		}
	}

	let layoutLabel = $derived(
		currentLayout === 'numbers' ? '123' : currentLayout === 'symbols' ? '#+=' : (mode === 'chat' ? 'Chat' : 'Terminal')
	);
</script>

{#if visible}
	<div
		class="keyboard-backdrop"
		onclick={handleBackdropClick}
		role="button"
		tabindex="-1"
		aria-hidden="true"
	></div>

	{#if previewKey}
		<div
			class="key-preview"
			style="left: {previewPosition.x}px; top: {previewPosition.y}px;"
			aria-hidden="true"
		>
			{isShifted && previewKey.value.length === 1 && previewKey.value.match(/[a-z]/)
				? previewKey.value.toUpperCase()
				: previewKey.display || previewKey.value}
		</div>
	{/if}

	<div
		bind:this={keyboardEl}
		class="keyboard-container"
		role="group"
		aria-label="Virtual keyboard"
		ontouchstart={handleTouchStart}
		ontouchend={handleTouchEnd}
	>
		{#each layout as row}
			<div class="keyboard-row">
				{#each row as key}
					<button
						class="key"
						class:shift-active={key.action === KeyAction.Shift && isShifted}
						data-width={key.width}
						data-action={key.action}
						onclick={() => handleKeyPress(key)}
						onmousedown={(e) => {
							if (key.action === KeyAction.Backspace) startRepeat(key);
							else showPreview(key, e);
						}}
						onmouseup={() => { stopRepeat(); hidePreview(); }}
						onmouseleave={() => { stopRepeat(); hidePreview(); }}
						ontouchstart={(e) => {
							e.preventDefault();
							if (key.action === KeyAction.Backspace) startRepeat(key);
						}}
						ontouchend={() => { stopRepeat(); hidePreview(); }}
						aria-label={key.display || key.value}
						aria-pressed={key.action === KeyAction.Shift ? isShifted : undefined}
						type="button"
					>
						<span class="key-content">
							{formatKeyDisplay(key.value, key.display)}
						</span>
					</button>
				{/each}
			</div>
		{/each}

		<div class="keyboard-footer">
			<span class="mode-indicator">{layoutLabel}</span>
		</div>
	</div>
{/if}

<style>
	/* Backdrop for tap-outside-to-dismiss */
	.keyboard-backdrop {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		z-index: 999;
	}

	/* Key preview popup (iOS style) */
	.key-preview {
		position: fixed;
		transform: translateX(-50%) translateY(-100%);
		z-index: 1001;
		
		/* Popup styling */
		min-width: 50px;
		height: 50px;
		display: flex;
		align-items: center;
		justify-content: center;
		
		/* Glassmorphism */
		background: rgba(40, 40, 50, 0.95);
		backdrop-filter: blur(12px);
		-webkit-backdrop-filter: blur(12px);
		border-radius: 12px;
		box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
		
		/* Text */
		color: white;
		font-size: 24px;
		font-weight: 500;
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
		
		/* Animation */
		animation: previewPop 0.15s ease-out;
		pointer-events: none;
	}

	@keyframes previewPop {
		from {
			transform: translateX(-50%) translateY(-100%) scale(0.8);
			opacity: 0;
		}
		to {
			transform: translateX(-50%) translateY(-100%) scale(1);
			opacity: 1;
		}
	}

	.keyboard-container {
		position: fixed;
		bottom: 0;
		left: 0;
		right: 0;
		z-index: 1000;
		
		/* Glassmorphism styling */
		background: rgba(20, 20, 25, 0.85);
		backdrop-filter: blur(24px);
		-webkit-backdrop-filter: blur(24px);
		border-top: 1px solid rgba(255, 255, 255, 0.08);
		border-radius: 20px 20px 0 0;
		
		/* Safe area padding */
		padding: 8px 4px;
		padding-bottom: max(8px, env(safe-area-inset-bottom, 0px));
		
		/* Animation */
		animation: slideUp 0.25s cubic-bezier(0.16, 1, 0.3, 1);
		
		/* Prevent selection */
		user-select: none;
		-webkit-user-select: none;
		touch-action: manipulation;
	}

	@keyframes slideUp {
		from {
			transform: translateY(100%);
			opacity: 0;
		}
		to {
			transform: translateY(0);
			opacity: 1;
		}
	}

	.keyboard-row {
		display: flex;
		justify-content: center;
		gap: 4px;
		margin-bottom: 6px;
	}

	.keyboard-row:last-of-type {
		margin-bottom: 0;
	}

	.key {
		display: flex;
		align-items: center;
		justify-content: center;
		min-width: 32px;
		height: 44px;
		padding: 0 10px;
		border: none;
		border-radius: 8px;
		
		/* Glass key appearance */
		background: rgba(255, 255, 255, 0.06);
		color: rgba(255, 255, 255, 0.9);
		font-size: 16px;
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
		font-weight: 400;
		
		/* Subtle glow on press */
		transition: all 0.1s ease;
		cursor: pointer;
		
		/* Prevent highlight */
		-webkit-tap-highlight-color: transparent;
	}

	.key:active {
		background: rgba(255, 255, 255, 0.15);
		transform: scale(0.96);
	}

	/* Wide keys (function keys) - using data attribute selector */
	.key[data-width="wide"] {
		min-width: 56px;
		font-size: 14px;
		font-weight: 500;
	}

	/* Extra wide keys (spacebar) */
	.key[data-width="extra-wide"] {
		min-width: 140px;
	}

	/* Action keys (special functions) */
	.key[data-action] {
		background: rgba(255, 255, 255, 0.1);
		font-weight: 500;
	}

	/* Active state (shift pressed) */
	.key.shift-active {
		background: rgba(255, 107, 53, 0.3);
		color: #ff6b35;
	}

	.key-content {
		display: flex;
		align-items: center;
		justify-content: center;
		line-height: 1;
	}

	/* Footer with mode indicator */
	.keyboard-footer {
		display: flex;
		justify-content: center;
		padding-top: 4px;
	}

	.mode-indicator {
		font-size: 10px;
		color: rgba(255, 255, 255, 0.3);
		text-transform: uppercase;
		letter-spacing: 1px;
		font-weight: 500;
	}

	/* Dark mode adjustments (already dark, but just in case) */
	:global(.light) .keyboard-container {
		background: rgba(255, 255, 255, 0.9);
		border-top-color: rgba(0, 0, 0, 0.1);
	}

	:global(.light) .key {
		background: rgba(0, 0, 0, 0.05);
		color: rgba(0, 0, 0, 0.8);
	}

	:global(.light) .key[data-action] {
		background: rgba(0, 0, 0, 0.1);
	}

	:global(.light) .key:active {
		background: rgba(0, 0, 0, 0.15);
	}

	/* Responsive adjustments */
	@media (orientation: landscape) and (max-height: 400px) {
		.keyboard-container {
			padding: 4px;
		}

		.keyboard-row {
			gap: 3px;
			margin-bottom: 4px;
		}

		.key {
			height: 36px;
			min-width: 28px;
			font-size: 14px;
			padding: 0 8px;
		}

		.key[data-width="wide"] {
			min-width: 48px;
			font-size: 12px;
		}

		.key[data-width="extra-wide"] {
			min-width: 100px;
		}

		.keyboard-footer {
			display: none;
		}
	}

	@media (min-width: 768px) {
		.key {
			min-width: 40px;
			height: 48px;
			font-size: 18px;
		}

		.key[data-width="wide"] {
			min-width: 64px;
		}

		.key[data-width="extra-wide"] {
			min-width: 180px;
		}
	}
</style>
