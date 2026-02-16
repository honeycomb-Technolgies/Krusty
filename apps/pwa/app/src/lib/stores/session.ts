import { writable, get } from 'svelte/store';
import { apiClient, streamChat, streamToolResult, type ContentBlock, type PlanItem, type StreamCallbacks } from '$api/client';
import { loadSessions } from './sessions';
import { setPlanItems, setPlanVisible } from './plan';
import { workspaceStore } from './workspace';

// Background state polling interval (for reconnection)
let statePollingInterval: ReturnType<typeof setInterval> | null = null;
const STATE_POLL_INTERVAL = 3000; // 3 seconds
const MAX_QUEUED_MESSAGES = 50;
const MAX_MESSAGE_CONTENT_LENGTH = 500_000; // 500KB

export interface ToolCall {
	id: string;
	name: string;
	description?: string;
	arguments?: Record<string, unknown>;
	output?: string;
	status: 'pending' | 'running' | 'success' | 'error' | 'awaiting_approval';
}

export interface ChatMessage {
	role: 'user' | 'assistant';
	content: string;
	thinking?: string;
	toolCalls?: ToolCall[];
	isQueued?: boolean;
}

export type SessionMode = 'build' | 'plan';
export type PermissionMode = 'supervised' | 'autonomous';

interface QueuedMessage {
	content: string;
	attachments: Attachment[];
}

interface SessionState {
	sessionId: string | null;
	title: string;
	mode: SessionMode;
	permissionMode: PermissionMode;
	messages: ChatMessage[];
	queuedMessages: QueuedMessage[];
	isLoading: boolean;
	isStreaming: boolean;
	isThinking: boolean;
	thinkingContent: string;
	thinkingEnabled: boolean;
	tokenCount: number;
	error: string | null;
}

function loadPermissionMode(): PermissionMode {
	try {
		const stored = localStorage.getItem('krusty-permission-mode');
		if (stored === 'supervised' || stored === 'autonomous') return stored;
	} catch { /* ignore */ }
	return 'supervised';
}

const initialState: SessionState = {
	sessionId: null,
	title: 'New Chat',
	mode: 'build',
	permissionMode: loadPermissionMode(),
	messages: [],
	queuedMessages: [],
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

const MAX_FILE_SIZE = 50 * 1024 * 1024; // 50MB

async function fileToBase64(file: File): Promise<string> {
	if (file.size > MAX_FILE_SIZE) {
		throw new Error(`File too large: ${(file.size / 1024 / 1024).toFixed(1)}MB exceeds 50MB limit`);
	}
	return new Promise((resolve, reject) => {
		const reader = new FileReader();
		reader.onload = () => {
			const result = reader.result as string;
			// Remove data URL prefix (e.g., "data:image/png;base64,")
			const parts = result.split(',');
			if (parts.length < 2) {
				reject(new Error('Invalid data URL format'));
				return;
			}
			const base64 = parts[1];
			resolve(base64);
		};
		reader.onerror = reject;
		reader.readAsDataURL(file);
	});
}

async function fileToText(file: File): Promise<string> {
	if (file.size > MAX_FILE_SIZE) {
		throw new Error(`File too large: ${(file.size / 1024 / 1024).toFixed(1)}MB exceeds 50MB limit`);
	}
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

// Mutable ref wrapper so callbacks can read/write the current assistant message
// across turn boundaries (onTurnComplete resets the message for the next turn)
interface AssistantMessageRef {
	current: ChatMessage;
}

function createStreamCallbacks(ref: AssistantMessageRef): StreamCallbacks {
	function updateLastAssistantMessage(updater?: (s: SessionState) => Partial<SessionState>) {
		sessionStore.update((s) => {
			const messages = [...s.messages];
			const lastIdx = messages.length - 1;
			if (messages[lastIdx]?.role === 'assistant') {
				messages[lastIdx] = { ...ref.current };
			} else {
				messages.push({ ...ref.current });
			}
			return { ...s, messages, ...updater?.(s) };
		});
	}

	function mapToolCalls(id: string, mapper: (tc: ToolCall) => ToolCall) {
		ref.current.toolCalls = ref.current.toolCalls?.map((tc) =>
			tc.id === id ? mapper(tc) : tc
		);
		updateLastAssistantMessage();
	}

	return {
		onTextDelta: (delta) => {
			ref.current.content += delta;
			updateLastAssistantMessage(() => ({ isLoading: false, isThinking: false }));
		},
		onThinkingDelta: (thinking) => {
			ref.current.thinking = (ref.current.thinking || '') + thinking;
			updateLastAssistantMessage(() => ({ isThinking: true, thinkingContent: ref.current.thinking || '' }));
		},
		onToolCallStart: (id, name) => {
			ref.current.toolCalls = [...(ref.current.toolCalls || []), { id, name, status: 'running' }];
			updateLastAssistantMessage();
		},
		onToolCallComplete: (id, _name, args) => {
			mapToolCalls(id, (tc) => ({ ...tc, arguments: args }));
		},
		onToolResult: (id, output, isError) => {
			mapToolCalls(id, (tc) => ({ ...tc, output, status: isError ? 'error' : 'success' }));
		},
		onToolOutputDelta: (id, delta) => {
			mapToolCalls(id, (tc) => ({ ...tc, output: (tc.output || '') + delta }));
		},
		onPlanUpdate: (items: PlanItem[]) => {
			setPlanItems(items);
		},
		onModeChange: (mode) => {
			sessionStore.update((s) => ({ ...s, mode: mode as SessionMode }));
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
			ref.current.toolCalls = [...(ref.current.toolCalls || []), planConfirmCall];
			updateLastAssistantMessage();
		},
		onTurnComplete: (_turn, hasMore) => {
			if (hasMore) {
				ref.current = { role: 'assistant', content: '', thinking: '', toolCalls: [] };
				sessionStore.update((s) => ({
					...s,
					messages: [...s.messages, { ...ref.current }]
				}));
			}
		},
		onToolApprovalRequired: (id, _name, args) => {
			mapToolCalls(id, (tc) => ({ ...tc, arguments: args, status: 'awaiting_approval' }));
		},
		onToolApproved: (id) => {
			mapToolCalls(id, (tc) => ({ ...tc, status: 'running' }));
		},
		onToolDenied: (id) => {
			mapToolCalls(id, (tc) => ({ ...tc, status: 'error', output: 'Denied by user' }));
		},
		onUsage: (promptTokens, _completionTokens) => {
			sessionStore.update((s) => ({ ...s, tokenCount: promptTokens }));
		},
		onTitleUpdate: (title) => {
			sessionStore.update((s) => ({ ...s, title }));
			loadSessions();
		},
		onFinish: (sessionId) => {
			const currentState = get(sessionStore);
			const queued = currentState.queuedMessages;

			const messages = currentState.messages.map(m =>
				m.isQueued ? { ...m, isQueued: false } : m
			);

			sessionStore.update((s) => ({
				...s,
				sessionId,
				messages,
				queuedMessages: [],
				isStreaming: false,
				isThinking: false,
				thinkingContent: ''
			}));
			loadSessions();

			if (queued.length > 0) {
				const combinedContent = queued.map(q => q.content).join('\n\n');
				const combinedAttachments = queued.flatMap(q => q.attachments);
				setTimeout(() => sendMessage(combinedContent, combinedAttachments), 50);
			}
		},
		onError: (error) => {
			sessionStore.update((s) => ({
				...s,
				isLoading: false,
				isStreaming: false,
				error
			}));
		}
	};
}

export async function sendMessage(content: string, attachments: Attachment[] = []) {
	const state = get(sessionStore);

	// Build display content for UI
	const displayContent = attachments.length > 0
		? `${content}\n\n[Attachments: ${attachments.map(a => a.file.name).join(', ')}]`
		: content;

	// Queue message if currently streaming
	if (state.isStreaming) {
		sessionStore.update((s) => {
			if (s.queuedMessages.length >= MAX_QUEUED_MESSAGES) {
				return { ...s, error: 'Message queue is full. Please wait for the current response to finish.' };
			}
			return {
				...s,
				queuedMessages: [...s.queuedMessages, { content, attachments }],
				messages: [...s.messages, { role: 'user', content: displayContent, isQueued: true }]
			};
		});
		return;
	}

	// Add user message
	sessionStore.update((s) => ({
		...s,
		messages: [...s.messages, { role: 'user', content: displayContent }],
		isLoading: true,
		isStreaming: true,
		error: null
	}));

	abortController = new AbortController();

	const pollingSessionId = state.sessionId;
	if (pollingSessionId) {
		startStatePolling(pollingSessionId);
	}

	try {
		const contentBlocks = attachments.length > 0
			? await buildContentBlocks(content, attachments)
			: undefined;

		const ref: AssistantMessageRef = {
			current: { role: 'assistant', content: '', thinking: '', toolCalls: [] }
		};

		await streamChat(
			{
				session_id: state.sessionId ?? undefined,
				message: content,
				content: contentBlocks,
				thinking_enabled: state.thinkingEnabled,
				permission_mode: state.permissionMode
			},
			createStreamCallbacks(ref),
			abortController.signal
		);
	} catch (err) {
		sessionStore.update((s) => ({
			...s,
			isLoading: false,
			isStreaming: false,
			error: err instanceof Error ? err.message : 'Unknown error'
		}));
	} finally {
		stopStatePolling();
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
	if (typeof content === 'string') return content;

	if (Array.isArray(content)) {
		return content
			.filter((block): block is { type: 'text'; text: string } => block?.type === 'text')
			.map((block) => block.text)
			.join('\n');
	}

	if (content && typeof content === 'object' && 'text' in content) {
		const textValue = (content as Record<string, unknown>).text;
		if (typeof textValue === 'string') return textValue;
	}

	return '';
}

export async function loadSession(sessionId: string, isRefresh = false) {
	sessionStore.update((s) => ({ ...s, isLoading: true }));

	try {
		const data = await apiClient.getSession(sessionId);
		const processedMessages = processStoredMessages(data.messages);
		sessionStore.update((s) => ({
			...s,
			sessionId: data.session.id,
			title: data.session.title || 'Untitled',
			messages: processedMessages,
			isLoading: false
		}));

		workspaceStore.initFromSession(data.session.id, data.session.working_dir ?? null);

		if (!isRefresh) {
			try {
				const state = await apiClient.getSessionState(sessionId);
				if (state.agent_state === 'streaming' || state.agent_state === 'tool_executing') {
					sessionStore.update((s) => ({ ...s, isStreaming: true }));
					startStatePolling(sessionId);
				}
			} catch {
				// State endpoint may not exist or session deleted
			}
		}
	} catch (err) {
		sessionStore.update((s) => ({
			...s,
			isLoading: false,
			error: err instanceof Error ? err.message : 'Failed to load session'
		}));
	}
}

function processStoredMessages(rawMessages: { role: string; content: unknown }[]): ChatMessage[] {
	const result: ChatMessage[] = [];

	const toolResults = new Map<string, { output: string; isError: boolean }>();

	for (const m of rawMessages) {
		const contentArray = Array.isArray(m.content) ? m.content : [];
		for (const block of contentArray) {
			if (!block || typeof block !== 'object') continue;
			if (block.type === 'tool_result' || 'tool_use_id' in block) {
				if (block.tool_use_id) {
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
	}

	for (const m of rawMessages) {
		const msg = parseStoredMessage(m, toolResults);

		const hasContent = msg.content.trim().length > 0;
		const hasThinking = (msg.thinking?.trim().length ?? 0) > 0;
		const hasToolCalls = (msg.toolCalls?.length ?? 0) > 0;

		if (hasContent || hasThinking || hasToolCalls) {
			result.push(msg);
		}
	}

	return result;
}

function parseStoredMessage(
	m: { role: string; content: unknown },
	toolResults?: Map<string, { output: string; isError: boolean }>
): ChatMessage {
	const role: 'user' | 'assistant' = m.role === 'user' || m.role === 'assistant' ? m.role : 'assistant';
	const msg: ChatMessage = {
		role,
		content: '',
		thinking: '',
		toolCalls: []
	};

	const contentArray = Array.isArray(m.content) ? m.content : [];

	for (const block of contentArray) {
		if (!block || typeof block !== 'object') continue;

		if (block.type === 'text' || ('text' in block && !block.type)) {
			if (msg.content.length < MAX_MESSAGE_CONTENT_LENGTH) {
				msg.content += (msg.content ? '\n' : '') + (block.text || '');
			}
		} else if (block.type === 'thinking' || 'thinking' in block) {
			const thinkingContent = block.thinking || '';
			msg.thinking = msg.thinking ? msg.thinking + '\n\n' + thinkingContent : thinkingContent;
		} else if (block.type === 'tool_use' || ('id' in block && 'name' in block && 'input' in block)) {
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
	}

	if (!msg.content && !msg.thinking && (!msg.toolCalls || msg.toolCalls.length === 0)) {
		msg.content = extractTextContent(m.content);
	}

	return msg;
}

export function clearSession() {
	sessionStore.set(initialState);
	workspaceStore.clear();
}

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
		loadSessions();
	} catch (err) {
		console.error('Failed to update session title:', err);
	}
}

export async function submitToolResult(toolCallId: string, result: string) {
	const state = get(sessionStore);
	if (!state.sessionId) {
		throw new Error('No active session');
	}

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

	abortController = new AbortController();
	startStatePolling(state.sessionId);

	const ref: AssistantMessageRef = {
		current: { role: 'assistant', content: '', thinking: '', toolCalls: [] }
	};

	sessionStore.update((s) => ({
		...s,
		messages: [...s.messages, ref.current]
	}));

	try {
		await streamToolResult(
			{
				session_id: state.sessionId,
				tool_call_id: toolCallId,
				result
			},
			createStreamCallbacks(ref),
			abortController.signal
		);
	} catch (err) {
		sessionStore.update((s) => ({
			...s,
			isLoading: false,
			isStreaming: false,
			error: err instanceof Error ? err.message : 'Unknown error'
		}));
	} finally {
		stopStatePolling();
	}
}

export function startStatePolling(sessionId: string) {
	stopStatePolling();

	statePollingInterval = setInterval(async () => {
		try {
			const state = await apiClient.getSessionState(sessionId);

			if (state.agent_state === 'idle') {
				stopStatePolling();
				sessionStore.update((s) => ({ ...s, isStreaming: false, isThinking: false }));
				await loadSession(sessionId, true);
			}
		} catch (err) {
			console.warn('State polling error:', err);
			stopStatePolling();
		}
	}, STATE_POLL_INTERVAL);
}

export function stopStatePolling() {
	if (statePollingInterval) {
		clearInterval(statePollingInterval);
		statePollingInterval = null;
	}
}

export function togglePermissionMode() {
	sessionStore.update((s) => {
		const newMode: PermissionMode = s.permissionMode === 'supervised' ? 'autonomous' : 'supervised';
		try { localStorage.setItem('krusty-permission-mode', newMode); } catch { /* ignore */ }
		return { ...s, permissionMode: newMode };
	});
}

export async function approveToolCall(toolCallId: string) {
	const state = get(sessionStore);
	if (!state.sessionId) return;
	await apiClient.submitToolApproval(state.sessionId, toolCallId, true);
}

export async function denyToolCall(toolCallId: string) {
	const state = get(sessionStore);
	if (!state.sessionId) return;
	await apiClient.submitToolApproval(state.sessionId, toolCallId, false);
}
