/**
 * Virtual Keyboard Layout Tests
 * 
 * Run with: npx vitest run
 * Install vitest: npm install -D vitest
 */

import { describe, it, expect } from 'vitest';
import { getLayout, formatKeyDisplay, getTerminalSequence, qwertyLayout, numbersLayout, symbolsLayout, terminalLayout, KeyAction } from './layouts';

describe('Keyboard Layouts', () => {
	describe('getLayout', () => {
		it('should return qwerty layout for qwerty type', () => {
			const layout = getLayout('qwerty');
			expect(layout).toBe(qwertyLayout);
		});

		it('should return numbers layout for numbers type', () => {
			const layout = getLayout('numbers');
			expect(layout).toBe(numbersLayout);
		});

		it('should return symbols layout for symbols type', () => {
			const layout = getLayout('symbols');
			expect(layout).toBe(symbolsLayout);
		});

		it('should return qwerty layout for unknown types', () => {
			// @ts-expect-error - testing invalid input
			const layout = getLayout('unknown');
			expect(layout).toBe(qwertyLayout);
		});
	});

	describe('formatKeyDisplay', () => {
		it('should return display value when provided', () => {
			expect(formatKeyDisplay('q', 'Q')).toBe('Q');
		});

		it('should return empty string for space', () => {
			expect(formatKeyDisplay('space')).toBe('');
		});

		it('should return value when no display provided', () => {
			expect(formatKeyDisplay('q')).toBe('q');
			expect(formatKeyDisplay('1')).toBe('1');
		});
	});

	describe('qwertyLayout', () => {
		it('should have 4 rows', () => {
			expect(qwertyLayout).toHaveLength(4);
		});

		it('should have shift key in row 3', () => {
			const row3 = qwertyLayout[2];
			expect(row3[0].action).toBe(KeyAction.Shift);
		});

		it('should have backspace key in row 3', () => {
			const row3 = qwertyLayout[2];
			const lastKey = row3[row3.length - 1];
			expect(lastKey.action).toBe(KeyAction.Backspace);
		});

		it('should have space key in row 4', () => {
			const row4 = qwertyLayout[3];
			const spaceKey = row4.find(k => k.action === KeyAction.Space);
			expect(spaceKey).toBeDefined();
			expect(spaceKey?.width).toBe('extra-wide');
		});

		it('should have enter key in row 4', () => {
			const row4 = qwertyLayout[3];
			const enterKey = row4.find(k => k.action === KeyAction.Enter);
			expect(enterKey).toBeDefined();
		});
	});

	describe('numbersLayout', () => {
		it('should have 4 rows', () => {
			expect(numbersLayout).toHaveLength(4);
		});

		it('should have digit keys in row 1', () => {
			const row1 = numbersLayout[0];
			expect(row1.map(k => k.value).join('')).toBe('1234567890');
		});
	});

	describe('terminalLayout', () => {
		it('should have 4 rows', () => {
			expect(terminalLayout).toHaveLength(4);
		});

		it('should have navigation keys in row 2', () => {
			const row2 = terminalLayout[1];
			const values = row2.map(k => k.value);
			expect(values).toContain('up');
			expect(values).toContain('down');
			expect(values).toContain('left');
			expect(values).toContain('right');
		});
	});

	describe('KeyAction enum', () => {
		it('should have correct values', () => {
			expect(KeyAction.Shift).toBe('shift');
			expect(KeyAction.Backspace).toBe('backspace');
			expect(KeyAction.Space).toBe('space');
			expect(KeyAction.Enter).toBe('enter');
			expect(KeyAction.SwitchNumbers).toBe('switch-numbers');
			expect(KeyAction.SwitchSymbols).toBe('switch-symbols');
			expect(KeyAction.SwitchQwerty).toBe('switch-qwerty');
		});
	});
});

describe('Terminal Key Sequences', () => {
	describe('getTerminalSequence', () => {
		it('should return up arrow escape sequence', () => {
			expect(getTerminalSequence('up')).toBe('\x1b[A');
		});

		it('should return down arrow escape sequence', () => {
			expect(getTerminalSequence('down')).toBe('\x1b[B');
		});

		it('should return right arrow escape sequence', () => {
			expect(getTerminalSequence('right')).toBe('\x1b[C');
		});

		it('should return left arrow escape sequence', () => {
			expect(getTerminalSequence('left')).toBe('\x1b[D');
		});

		it('should return escape character for escape key', () => {
			expect(getTerminalSequence('escape')).toBe('\x1b');
		});

		it('should return tab character for tab key', () => {
			expect(getTerminalSequence('tab')).toBe('\t');
		});

		it('should return backspace character', () => {
			expect(getTerminalSequence('backspace')).toBe('\x7f');
		});

		it('should return carriage return for enter', () => {
			expect(getTerminalSequence('enter')).toBe('\r');
		});

		it('should return space for space key', () => {
			expect(getTerminalSequence('space')).toBe(' ');
		});

		it('should return empty string for ctrl', () => {
			expect(getTerminalSequence('ctrl')).toBe('');
		});

		it('should return unknown keys as-is', () => {
			expect(getTerminalSequence('ls')).toBe('ls');
			expect(getTerminalSequence('cd')).toBe('cd');
			expect(getTerminalSequence('test')).toBe('test');
		});
	});
});

/**
 * Integration Test: Terminal Key Sequence Generation
 * 
 * This test verifies that terminal keys generate the correct ANSI escape sequences
 * when combined with the terminalLayout configuration.
 */
describe('Terminal Layout Integration', () => {
	describe('Terminal key presses generate correct sequences', () => {
		it('should generate escape sequence for up arrow key', () => {
			// Find the up key in terminal layout
			const upKey = terminalLayout[1].find(k => k.value === 'up');
			expect(upKey).toBeDefined();
			
			// Verify the escape sequence
			const sequence = getTerminalSequence(upKey!.value);
			expect(sequence).toBe('\x1b[A');
		});

		it('should generate escape sequence for down arrow key', () => {
			const downKey = terminalLayout[1].find(k => k.value === 'down');
			expect(downKey).toBeDefined();
			
			const sequence = getTerminalSequence(downKey!.value);
			expect(sequence).toBe('\x1b[B');
		});

		it('should generate escape sequence for left arrow key', () => {
			const leftKey = terminalLayout[1].find(k => k.value === 'left');
			expect(leftKey).toBeDefined();
			
			const sequence = getTerminalSequence(leftKey!.value);
			expect(sequence).toBe('\x1b[D');
		});

		it('should generate escape sequence for right arrow key', () => {
			const rightKey = terminalLayout[1].find(k => k.value === 'right');
			expect(rightKey).toBeDefined();
			
			const sequence = getTerminalSequence(rightKey!.value);
			expect(sequence).toBe('\x1b[C');
		});

		it('should return plain text for command keys (ls, cd, etc.)', () => {
			const commandKeys = ['ls', 'cd', 'pwd', 'cat', 'grep', 'git'];
			
			commandKeys.forEach(cmd => {
				const key = terminalLayout[2].find(k => k.value === cmd);
				expect(key).toBeDefined();
				
				// Command keys should return themselves as text
				const result = getTerminalSequence(key!.value);
				expect(result).toBe(cmd);
			});
		});

		it('should handle special action keys correctly', () => {
			// Backspace should return delete character
			const backspaceKey = terminalLayout[2].find(k => k.action === KeyAction.Backspace);
			expect(backspaceKey).toBeDefined();
			expect(getTerminalSequence(backspaceKey!.value)).toBe('\x7f');

			// Space should return space character
			const spaceKey = terminalLayout[3].find(k => k.action === KeyAction.Space);
			expect(spaceKey).toBeDefined();
			expect(getTerminalSequence(spaceKey!.value)).toBe(' ');

			// Enter should return carriage return
			const enterKey = terminalLayout[3].find(k => k.action === KeyAction.Enter);
			expect(enterKey).toBeDefined();
			expect(getTerminalSequence(enterKey!.value)).toBe('\r');
		});
	});

	describe('Navigation flow simulation', () => {
		it('should generate correct sequence for up-down-left-right navigation', () => {
			// Simulating: up, right, down, left (clockwise)
			const navKeys = ['up', 'right', 'down', 'left'];
			const expectedSequences = ['\x1b[A', '\x1b[C', '\x1b[B', '\x1b[D'];
			
			navKeys.forEach((key, index) => {
				const sequence = getTerminalSequence(key);
				expect(sequence).toBe(expectedSequences[index]);
			});
		});

		it('should generate correct sequence for command + argument', () => {
			// Simulating: cd .. (cd command with argument)
			const cdSequence = getTerminalSequence('cd');
			const dotsSequence = getTerminalSequence('..');
			
			expect(cdSequence).toBe('cd');
			expect(dotsSequence).toBe('..');
		});
	});
});
