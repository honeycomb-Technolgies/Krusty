/**
 * Terminal-specific keyboard layout and escape sequences
 */

import { KeyAction, type KeyConfig } from './types';

/**
 * Terminal-specific keys (for when keyboard is in terminal mode)
 * Provides quick access to common terminal shortcuts
 */
export const terminalLayout: KeyConfig[][] = [
	// Row 1 - Function and escape
	[
		{ value: 'ctrl', display: 'Ctrl', width: 'wide' },
		{ value: 'escape', display: 'Esc' },
		{ value: 'tab', display: 'Tab' },
		{ value: ':' }, { value: ' ' }
	],
	// Row 2 - Navigation
	[
		{ value: 'up', display: '↑' },
		{ value: 'down', display: '↓' },
		{ value: 'left', display: '←' },
		{ value: 'right', display: '→' },
		{ value: 'c', display: 'C' }
	],
	// Row 3 - Common commands
	[
		{ value: 'ls', display: 'ls' },
		{ value: 'cd', display: 'cd' },
		{ value: 'pwd', display: 'pwd' },
		{ value: 'cat', display: 'cat' },
		{ value: 'grep', display: 'grep' },
		{ value: 'git', display: 'git' },
		{ value: 'backspace', display: '⌫', action: KeyAction.Backspace }
	],
	// Row 4
	[
		{ value: '123', display: '123', action: KeyAction.SwitchNumbers, width: 'wide' },
		{ value: '|' },
		{ value: 'space', display: 'space', action: KeyAction.Space, width: 'extra-wide' },
		{ value: 'enter', display: '⏎', action: KeyAction.Enter }
	]
];

/**
 * Terminal key to xterm/ANSI escape sequence mapping
 */
export function getTerminalSequence(key: string): string {
	const sequences: Record<string, string> = {
		// Navigation
		up: '\x1b[A',
		down: '\x1b[B',
		right: '\x1b[C',
		left: '\x1b[D',
		
		// Function keys
		escape: '\x1b',
		tab: '\t',
		
		// Ctrl combinations (simplified - would need modifier handling)
		ctrl: '',
		
		// Special
		backspace: '\x7f',
		enter: '\r',
		space: ' ',
	};
	
	return sequences[key] || key;
}
