import { writable, get } from 'svelte/store';
import { apiClient, streamChat, streamToolResult, type ContentBlock, type PlanItem } from '$api/client';
import { loadSessions } from './sessions';
import { setPlanItems, setPlanVisible } from './plan';
import { workspaceStore } from './workspace';

// Background state polling interval (for reconnection)
let statePollingInterval: ReturnType<typeof setInterval> | null = null;
const STATE_POLL_INTERVAL = 3000; // 3 seconds

export interface ToolCall {
	id: string;
	name: string;
	description?: string;
	arguments?: Record<string, unknown>;
	output?: string;
	status: 'pending' | 'running' | 'success' | 'error';
}

export interface ChatMessage {
	role: 'user' | 'assistant';
	content: string;
	thinking?: string;
	toolCalls?: ToolCall[];
}

export type SessionMode = 'build' | 'plan';

interface SessionState {
	sessionId: string | null;
	title: string;
	mode: SessionMode;
	messages: ChatMessage[];
	isLoading: boolean;
	isStreaming: boolean;
	isThinking: boolean;
	thinkingContent: string;
	thinkingEnabled: boolean;
	tokenCount: number;
	error: string | null;
}

const initialState: SessionState = {
	sessionId: null,
	title: 'New Chat',
	mode: 'build',
	messages: [],
	isLoading: false,
	isStreaming: false,
	isThinking: false,
	thinkingContent: '',
	thinkingEnabled: true,
	tokenCount: 0,
	error: null
};

export const sessionStore = writable<SessionState>(initialState);

let abortController: AbortController | null = null;

export interface Attachment {
	file: File;
	type: 'image' | 'file';
}

async function fileToBase64(file: File): Promise<string> {
	return new Promise((resolve, reject) => {
		const reader = new FileReader();
		reader.onload = () => {
			const result = reader.result as string;
			// Remove data URL prefix (e.g., "data:image/png;base64,")
			const base64 = result.split(',')[1];
			resolve(base64);
		};
		reader.onerror = reject;
		reader.readAsDataURL(file);
	});
}

async function fileToText(file: File): Promise<string> {
	return new Promise((resolve, reject) => {
		const reader = new FileReader();
		reader.onload = () => resolve(reader.result as string);
		reader.onerror = reject;
		reader.readAsText(file);
	});
}

async function buildContentBlocks(text: string, attachments: Attachment[]): Promise<ContentBlock[]> {
	const blocks: ContentBlock[] = [];

	// Process images first
	for (const att of attachments) {
		if (att.type === 'image') {
			const base64 = await fileToBase64(att.file);
			blocks.push({
				type: 'image',
				source: {
					type: 'base64',
					media_type: att.file.type || 'image/png',
					data: base64
				}
			});
		}
	}

	// Process text files - prepend their content to the message
	let fileContent = '';
	for (const att of attachments) {
		if (att.type === 'file') {
			const content = await fileToText(att.file);
			fileContent += `\n\n--- ${att.file.name} ---\n${content}`;
		}
	}

	// Add text block
	const fullText = fileContent ? `${text}\n${fileContent}` : text;
	blocks.push({ type: 'text', text: fullText });

	return blocks;
}

export async function sendMessage(content: string, attachments: Attachment[] = []) {
	const state = get(sessionStore);

	// Build display content for UI
	const displayContent = attachments.length > 0
		? `${content}\n\n[Attachments: ${attachments.map(a => a.file.name).join(', ')}]`
		: content;

	// Add user message
	sessionStore.update((s) => ({
		...s,
		messages: [...s.messages, { role: 'user', content: displayContent }],
		isLoading: true,
		isStreaming: true,
		error: null
	}));

	// Create abort controller for cancellation
	abortController = new AbortController();

	try {
		// Build content blocks if we have attachments
		const contentBlocks = attachments.length > 0
			? await buildContentBlocks(content, attachments)
			: undefined;

		let assistantMessage: ChatMessage = { role: 'assistant', content: '', thinking: '', toolCalls: [] };

		await streamChat(
			{
				session_id: state.sessionId ?? undefined,
				message: content,
				content: contentBlocks,
				thinking_enabled: state.thinkingEnabled
			},
			{
				onTextDelta: (delta) => {
					assistantMessage.content += delta;
					sessionStore.update((s) => {
						const messages = [...s.messages];
						const lastIdx = messages.length - 1;
						if (messages[lastIdx]?.role === 'assistant') {
							messages[lastIdx] = { ...assistantMessage };
						} else {
							messages.push({ ...assistantMessage });
						}
						return { ...s, messages, isLoading: false, isThinking: false };
					});
				},
				onThinkingDelta: (thinking) => {
					assistantMessage.thinking = (assistantMessage.thinking || '') + thinking;
					sessionStore.update((s) => {
						const messages = [...s.messages];
						const lastIdx = messages.length - 1;
						if (messages[lastIdx]?.role === 'assistant') {
							messages[lastIdx] = { ...assistantMessage };
						} else {
							messages.push({ ...assistantMessage });
						}
						return { ...s, messages, isThinking: true, thinkingContent: assistantMessage.thinking || '' };
					});
				},
				onToolCallStart: (id, name) => {
					const toolCall: ToolCall = { id, name, status: 'running' };
					assistantMessage.toolCalls = [...(assistantMessage.toolCalls || []), toolCall];
					sessionStore.update((s) => {
						const messages = [...s.messages];
						const lastIdx = messages.length - 1;
						if (messages[lastIdx]?.role === 'assistant') {
							messages[lastIdx] = { ...assistantMessage };
						}
						return { ...s, messages };
					});
				},
				onToolCallComplete: (id, name, args) => {
					const toolCalls = assistantMessage.toolCalls?.map((tc) =>
						tc.id === id ? { ...tc, arguments: args } : tc
					);
					assistantMessage.toolCalls = toolCalls;
					// Update store so UI receives the arguments
					sessionStore.update((s) => {
						const messages = [...s.messages];
						const lastIdx = messages.length - 1;
						if (messages[lastIdx]?.role === 'assistant') {
							messages[lastIdx] = { ...assistantMessage };
						}
						return { ...s, messages };
					});
				},
				onToolResult: (id, output, isError) => {
					const toolCalls = assistantMessage.toolCalls?.map((tc): ToolCall =>
						tc.id === id
							? { ...tc, output, status: isError ? 'error' : 'success' }
							: tc
					);
					assistantMessage.toolCalls = toolCalls;
					sessionStore.update((s) => {
						const messages = [...s.messages];
						const lastIdx = messages.length - 1;
						if (messages[lastIdx]?.role === 'assistant') {
							messages[lastIdx] = { ...assistantMessage };
						}
						return { ...s, messages };
					});
				},
				onPlanUpdate: (items: PlanItem[]) => {
					setPlanItems(items);
				},
				onModeChange: (mode) => {
					sessionStore.update((s) => ({
						...s,
						mode: mode as SessionMode
					}));
					if (mode === 'plan') {
						setPlanVisible(true);
					}
				},
				onPlanComplete: (toolCallId, title, taskCount) => {
					const planConfirmCall: ToolCall = {
						id: toolCallId,
						name: 'PlanConfirm',
						arguments: { title, task_count: taskCount },
						status: 'pending'
					};
					assistantMessage.toolCalls = [...(assistantMessage.toolCalls || []), planConfirmCall];
					sessionStore.update((s) => {
						const messages = [...s.messages];
						const lastIdx = messages.length - 1;
						if (messages[lastIdx]?.role === 'assistant') {
							messages[lastIdx] = { ...assistantMessage };
						}
						return { ...s, messages };
					});
				},
				onUsage: (promptTokens, completionTokens) => {
					// promptTokens is the context size for this request - shows how full context is
					sessionStore.update((s) => ({
						...s,
						tokenCount: promptTokens // Context size, not cumulative
					}));
				},
				onTitleUpdate: (title) => {
					// Haiku generated a better title
					sessionStore.update((s) => ({ ...s, title }));
					// Refresh sessions list to show updated title in sidebar
					loadSessions();
				},
				onFinish: (sessionId) => {
					sessionStore.update((s) => ({
						...s,
						sessionId,
						isStreaming: false,
						isThinking: false,
						thinkingContent: ''
					}));
					// Refresh sessions list to show new/updated session
					loadSessions();
				},
				onError: (error) => {
					sessionStore.update((s) => ({
						...s,
						isLoading: false,
						isStreaming: false,
						error
					}));
				}
			},
			abortController.signal
		);
	} catch (err) {
		sessionStore.update((s) => ({
			...s,
			isLoading: false,
			isStreaming: false,
			error: err instanceof Error ? err.message : 'Unknown error'
		}));
	}
}

export function stopGeneration() {
	abortController?.abort();
	sessionStore.update((s) => ({
		...s,
		isLoading: false,
		isStreaming: false,
		isThinking: false
	}));
}

function extractTextContent(content: unknown): string {
	// Handle string content directly
	if (typeof content === 'string') return content;

	// Handle array of content blocks (Claude API format)
	if (Array.isArray(content)) {
		return content
			.filter((block): block is { type: 'text'; text: string } => block?.type === 'text')
			.map((block) => block.text)
			.join('\n');
	}

	// Handle object with text property
	if (content && typeof content === 'object' && 'text' in content) {
		return (content as { text: string }).text;
	}

	return '';
}

export async function loadSession(sessionId: string) {
	sessionStore.update((s) => ({ ...s, isLoading: true }));

	try {
		const data = await apiClient.getSession(sessionId);
		// Process messages with tool result association
		const processedMessages = processStoredMessages(data.messages);
		sessionStore.update((s) => ({
			...s,
			sessionId: data.session.id,
			title: data.session.title || 'Untitled',
			messages: processedMessages,
			isLoading: false
		}));

		// Update workspace store - this syncs IDE, terminal, and other tabs
		workspaceStore.initFromSession(data.session.id, data.session.working_dir ?? null);
	} catch (err) {
		sessionStore.update((s) => ({
			...s,
			isLoading: false,
			error: err instanceof Error ? err.message : 'Failed to load session'
		}));
	}
}

// Process stored messages, associating tool results with their tool calls
function processStoredMessages(rawMessages: { role: string; content: unknown }[]): ChatMessage[] {
	const result: ChatMessage[] = [];

	// First pass: parse all messages and collect tool results
	const toolResults = new Map<string, { output: string; isError: boolean }>();

	for (const m of rawMessages) {
		const contentArray = Array.isArray(m.content) ? m.content : [];
		for (const block of contentArray) {
			if (!block || typeof block !== 'object') continue;
			// Collect tool results by their tool_use_id
			if (block.type === 'tool_result' || 'tool_use_id' in block) {
				const output = typeof block.output === 'string'
					? block.output
					: typeof block.content === 'string'
						? block.content
						: JSON.stringify(block.output || block.content || '');
				toolResults.set(block.tool_use_id, {
					output,
					isError: block.is_error === true
				});
			}
		}
	}

	// Second pass: build messages with tool results associated
	for (const m of rawMessages) {
		const msg = parseStoredMessage(m, toolResults);

		// Filter out empty messages (pure tool_result messages with no other content)
		const hasContent = msg.content.trim().length > 0;
		const hasThinking = (msg.thinking?.trim().length ?? 0) > 0;
		const hasToolCalls = (msg.toolCalls?.length ?? 0) > 0;

		if (hasContent || hasThinking || hasToolCalls) {
			result.push(msg);
		}
	}

	return result;
}

// Parse a stored message into ChatMessage format
function parseStoredMessage(
	m: { role: string; content: unknown },
	toolResults?: Map<string, { output: string; isError: boolean }>
): ChatMessage {
	const msg: ChatMessage = {
		role: m.role as 'user' | 'assistant',
		content: '',
		thinking: '',
		toolCalls: []
	};

	// Content can be array of blocks or a string
	const contentArray = Array.isArray(m.content) ? m.content : [];

	for (const block of contentArray) {
		if (!block || typeof block !== 'object') continue;

		// Text block
		if (block.type === 'text' || ('text' in block && !block.type)) {
			msg.content += (msg.content ? '\n' : '') + (block.text || '');
		}
		// Thinking block (concatenate if multiple)
		else if (block.type === 'thinking' || 'thinking' in block) {
			const thinkingContent = block.thinking || '';
			msg.thinking = msg.thinking ? msg.thinking + '\n\n' + thinkingContent : thinkingContent;
		}
		// Tool use block (assistant requesting tool)
		else if (block.type === 'tool_use' || ('id' in block && 'name' in block && 'input' in block)) {
			msg.toolCalls = msg.toolCalls || [];
			const toolResult = toolResults?.get(block.id);
			msg.toolCalls.push({
				id: block.id,
				name: block.name,
				arguments: block.input,
				output: toolResult?.output,
				status: toolResult?.isError ? 'error' : 'success' as const
			});
		}
		// Tool result blocks are collected separately in processStoredMessages
	}

	// Fallback: if no structured content, try extractTextContent
	if (!msg.content && !msg.thinking && (!msg.toolCalls || msg.toolCalls.length === 0)) {
		msg.content = extractTextContent(m.content);
	}

	return msg;
}

export function clearSession() {
	sessionStore.set(initialState);
	workspaceStore.clear();
}

// Initialize a new session (after creating via API)
export function initSession(sessionId: string, title: string) {
	sessionStore.set({
		...initialState,
		sessionId,
		title
	});
}

export function toggleThinking() {
	sessionStore.update((s) => ({
		...s,
		thinkingEnabled: !s.thinkingEnabled
	}));
}

export function setTitle(title: string) {
	sessionStore.update((s) => ({ ...s, title }));
}

export function setMode(mode: SessionMode) {
	sessionStore.update((s) => ({ ...s, mode }));
}

export async function updateSessionTitle(sessionId: string, title: string) {
	try {
		await apiClient.updateSession(sessionId, { title });
		sessionStore.update((s) => ({ ...s, title }));
		loadSessions(); // Refresh sidebar
	} catch (err) {
		console.error('Failed to update session title:', err);
	}
}

export async function submitToolResult(toolCallId: string, result: string) {
	const state = get(sessionStore);
	if (!state.sessionId) {
		throw new Error('No active session');
	}

	// Update tool call status to success
	sessionStore.update((s) => ({
		...s,
		messages: s.messages.map((m) => ({
			...m,
			toolCalls: m.toolCalls?.map((tc) =>
				tc.id === toolCallId ? { ...tc, output: result, status: 'success' as const } : tc
			)
		})),
		isStreaming: true,
		isLoading: true
	}));

	// Create abort controller for cancellation
	abortController = new AbortController();

	// Create a NEW assistant message for this response - don't merge with previous
	let assistantMessage: ChatMessage = { role: 'assistant', content: '', thinking: '', toolCalls: [] };

	// Add it to messages immediately so streaming updates go to the right place
	sessionStore.update((s) => ({
		...s,
		messages: [...s.messages, assistantMessage]
	}));

	try {
		await streamToolResult(
			{
				session_id: state.sessionId,
				tool_call_id: toolCallId,
				result
			},
			{
				onTextDelta: (delta) => {
					assistantMessage.content += delta;
					sessionStore.update((s) => {
						const messages = [...s.messages];
						const lastIdx = messages.length - 1;
						if (messages[lastIdx]?.role === 'assistant') {
							messages[lastIdx] = { ...assistantMessage };
						} else {
							messages.push({ ...assistantMessage });
						}
						return { ...s, messages, isLoading: false, isThinking: false };
					});
				},
				onThinkingDelta: (thinking) => {
					assistantMessage.thinking = (assistantMessage.thinking || '') + thinking;
					sessionStore.update((s) => {
						const messages = [...s.messages];
						const lastIdx = messages.length - 1;
						if (messages[lastIdx]?.role === 'assistant') {
							messages[lastIdx] = { ...assistantMessage };
						} else {
							messages.push({ ...assistantMessage });
						}
						return { ...s, messages, isThinking: true, thinkingContent: assistantMessage.thinking || '' };
					});
				},
				onToolCallStart: (id, name) => {
					const toolCall: ToolCall = { id, name, status: 'running' };
					assistantMessage.toolCalls = [...(assistantMessage.toolCalls || []), toolCall];
					sessionStore.update((s) => {
						const messages = [...s.messages];
						const lastIdx = messages.length - 1;
						if (messages[lastIdx]?.role === 'assistant') {
							messages[lastIdx] = { ...assistantMessage };
						}
						return { ...s, messages };
					});
				},
				onToolCallComplete: (id, name, args) => {
					const toolCalls = assistantMessage.toolCalls?.map((tc) =>
						tc.id === id ? { ...tc, arguments: args } : tc
					);
					assistantMessage.toolCalls = toolCalls;
					// Update store so UI receives the arguments
					sessionStore.update((s) => {
						const messages = [...s.messages];
						const lastIdx = messages.length - 1;
						if (messages[lastIdx]?.role === 'assistant') {
							messages[lastIdx] = { ...assistantMessage };
						}
						return { ...s, messages };
					});
				},
				onToolResult: (id, output, isError) => {
					const toolCalls = assistantMessage.toolCalls?.map((tc): ToolCall =>
						tc.id === id
							? { ...tc, output, status: isError ? 'error' : 'success' }
							: tc
					);
					assistantMessage.toolCalls = toolCalls;
					sessionStore.update((s) => {
						const messages = [...s.messages];
						const lastIdx = messages.length - 1;
						if (messages[lastIdx]?.role === 'assistant') {
							messages[lastIdx] = { ...assistantMessage };
						}
						return { ...s, messages };
					});
				},
				onPlanUpdate: (items: PlanItem[]) => {
					setPlanItems(items);
				},
				onModeChange: (mode) => {
					sessionStore.update((s) => ({
						...s,
						mode: mode as SessionMode
					}));
					if (mode === 'plan') {
						setPlanVisible(true);
					}
				},
				onPlanComplete: (toolCallId, title, taskCount) => {
					const planConfirmCall: ToolCall = {
						id: toolCallId,
						name: 'PlanConfirm',
						arguments: { title, task_count: taskCount },
						status: 'pending'
					};
					assistantMessage.toolCalls = [...(assistantMessage.toolCalls || []), planConfirmCall];
					sessionStore.update((s) => {
						const messages = [...s.messages];
						const lastIdx = messages.length - 1;
						if (messages[lastIdx]?.role === 'assistant') {
							messages[lastIdx] = { ...assistantMessage };
						}
						return { ...s, messages };
					});
				},
				onUsage: (promptTokens, completionTokens) => {
					// promptTokens is the context size for this request - shows how full context is
					sessionStore.update((s) => ({
						...s,
						tokenCount: promptTokens // Context size, not cumulative
					}));
				},
				onTitleUpdate: (title) => {
					sessionStore.update((s) => ({ ...s, title }));
					loadSessions();
				},
				onFinish: (sessionId) => {
					sessionStore.update((s) => ({
						...s,
						sessionId,
						isStreaming: false,
						isThinking: false,
						thinkingContent: ''
					}));
					loadSessions();
				},
				onError: (error) => {
					sessionStore.update((s) => ({
						...s,
						isLoading: false,
						isStreaming: false,
						error
					}));
				}
			},
			abortController.signal
		);
	} catch (err) {
		sessionStore.update((s) => ({
			...s,
			isLoading: false,
			isStreaming: false,
			error: err instanceof Error ? err.message : 'Unknown error'
		}));
	}
}

/**
 * Start polling for agent state changes (used when reconnecting to active session)
 * When agent becomes idle, reload messages to get final output
 */
export function startStatePolling(sessionId: string) {
	stopStatePolling(); // Clear any existing polling

	statePollingInterval = setInterval(async () => {
		try {
			const state = await apiClient.getSessionState(sessionId);

			if (state.agent_state === 'idle') {
				// Agent finished - stop polling and reload messages
				stopStatePolling();
				await loadSession(sessionId);
			}
		} catch (err) {
			// Session may have been deleted, stop polling
			console.warn('State polling error:', err);
			stopStatePolling();
		}
	}, STATE_POLL_INTERVAL);
}

/**
 * Stop polling for agent state changes
 */
export function stopStatePolling() {
	if (statePollingInterval) {
		clearInterval(statePollingInterval);
		statePollingInterval = null;
	}
}
