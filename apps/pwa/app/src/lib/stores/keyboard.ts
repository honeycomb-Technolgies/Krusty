import { writable } from 'svelte/store';

export type KeyboardSource = 'none' | 'virtual' | 'native';

interface KeyboardState {
	active: boolean;
	height: number;
	source: KeyboardSource;
}

const initial: KeyboardState = { active: false, height: 0, source: 'none' };

export const keyboardStore = writable<KeyboardState>(initial);

function syncCssVariables(state: KeyboardState) {
	if (typeof document === 'undefined') return;
	document.documentElement.style.setProperty('--keyboard-height', `${state.height}px`);
	if (state.active) {
		document.documentElement.setAttribute('data-keyboard-active', '');
	} else {
		document.documentElement.removeAttribute('data-keyboard-active');
	}
}

export function setVirtualKeyboardHeight(height: number) {
	keyboardStore.update((s) => {
		// Virtual keyboard closing
		if (height === 0 && s.source === 'virtual') {
			const next = { active: false, height: 0, source: 'none' as KeyboardSource };
			syncCssVariables(next);
			return next;
		}
		// Virtual keyboard opening/resizing
		if (height > 0) {
			const next = { active: true, height, source: 'virtual' as KeyboardSource };
			syncCssVariables(next);
			return next;
		}
		return s;
	});
}

export function setNativeKeyboardState(active: boolean, height: number) {
	keyboardStore.update((s) => {
		// Don't override virtual keyboard state
		if (s.source === 'virtual' && s.active) return s;
		const next: KeyboardState = active
			? { active: true, height, source: 'native' }
			: { active: false, height: 0, source: 'none' };
		syncCssVariables(next);
		return next;
	});
}
