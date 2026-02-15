<script lang="ts">
	import { onDestroy } from 'svelte';
	import { browser } from '$app/environment';
	import { ideStore, saveFile, updateFileContent } from '$stores/ide';

	let editorContainer: HTMLDivElement;
	let editorView: any = null;
	let currentFilePath: string | null = null;

	// Track the active file
	let activeFilePath = $derived($ideStore.activeFilePath);
	let activeFile = $derived($ideStore.openFiles.find((f) => f.path === $ideStore.activeFilePath));

	// Initialize or update editor when file changes
	$effect(() => {
		if (!browser || !editorContainer) return;

		if (activeFilePath !== currentFilePath) {
			currentFilePath = activeFilePath;
			if (activeFilePath && activeFile) {
				initEditor(activeFilePath, activeFile.content);
			} else {
				destroyEditor();
			}
		}
	});

	async function initEditor(filePath: string, initialContent: string) {
		destroyEditor();

		const { EditorView, basicSetup } = await import('codemirror');
		const { EditorState } = await import('@codemirror/state');
		const { oneDark } = await import('@codemirror/theme-one-dark');
		const { keymap } = await import('@codemirror/view');
		const { defaultKeymap, indentWithTab } = await import('@codemirror/commands');

		const langExtension = await getLanguageExtension(filePath);

		const state = EditorState.create({
			doc: initialContent,
			extensions: [
				basicSetup,
				oneDark,
				keymap.of([...defaultKeymap, indentWithTab]),
				langExtension,
				EditorView.updateListener.of((update) => {
					if (update.docChanged && currentFilePath) {
						const newContent = update.state.doc.toString();
						updateFileContent(currentFilePath, newContent);
					}
				}),
				EditorView.theme({
					'&': { height: '100%' },
					'.cm-scroller': {
						fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
						fontSize: '14px'
					}
				})
			]
		});

		editorView = new EditorView({
			state,
			parent: editorContainer
		});

		editorView.focus();
	}

	function destroyEditor() {
		if (editorView) {
			editorView.destroy();
			editorView = null;
		}
	}

	async function getLanguageExtension(filename: string) {
		const ext = filename.split('.').pop()?.toLowerCase();

		switch (ext) {
			case 'js':
			case 'jsx':
			case 'ts':
			case 'tsx':
			case 'mjs':
			case 'cjs':
				const { javascript } = await import('@codemirror/lang-javascript');
				return javascript({ jsx: ext?.includes('x'), typescript: ext?.includes('ts') });
			case 'json':
				const { json } = await import('@codemirror/lang-json');
				return json();
			case 'html':
			case 'svelte':
			case 'vue':
				const { html } = await import('@codemirror/lang-html');
				return html();
			case 'css':
			case 'scss':
			case 'less':
				const { css } = await import('@codemirror/lang-css');
				return css();
			case 'md':
			case 'markdown':
				const { markdown } = await import('@codemirror/lang-markdown');
				return markdown();
			case 'py':
			case 'python':
				const { python } = await import('@codemirror/lang-python');
				return python();
			case 'rs':
				const { rust } = await import('@codemirror/lang-rust');
				return rust();
			default:
				return [];
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if ((e.metaKey || e.ctrlKey) && e.key === 's') {
			e.preventDefault();
			saveFile();
		}
	}

	// Expose insert function for SymbolBar
	export function insertAtCursor(text: string) {
		if (!editorView) return;

		const { state } = editorView;
		const { from, to } = state.selection.main;

		editorView.dispatch({
			changes: { from, to, insert: text },
			selection: { anchor: from + text.length }
		});

		editorView.focus();
	}

	onDestroy(() => {
		destroyEditor();
	});
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="h-full bg-[#282c34]">
	<div bind:this={editorContainer} class="h-full"></div>
</div>

<style>
	:global(.cm-editor) {
		height: 100%;
	}

	:global(.cm-focused) {
		outline: none !important;
	}
</style>
