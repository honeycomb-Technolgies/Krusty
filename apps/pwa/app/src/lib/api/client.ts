const API_BASE = (import.meta.env.VITE_API_BASE || '/api').replace(/\/+$/, '');

const STREAM_ACTIVITY_TIMEOUT = 30_000; // 30 seconds
const STREAM_CHECK_INTERVAL = 5_000; // Check every 5 seconds

// ============================================================================
// Type Definitions
// ============================================================================

/** Session info returned from API */
export interface SessionResponse {
	id: string;
	title: string;
	token_count?: number | null;
	working_dir: string | null;
	parent_session_id: string | null;
	updated_at: string;
}

/** Message content block */
export interface MessageContent {
	type: 'text' | 'tool_use' | 'tool_result' | 'thinking';
	text?: string;
	id?: string;
	name?: string;
	input?: Record<string, unknown>;
	tool_use_id?: string;
	content?: string;
	thinking?: string;
}

/** Message in a session */
export interface MessageResponse {
	role: 'user' | 'assistant';
	content: MessageContent[];
}

/** Session with messages */
export interface SessionWithMessagesResponse {
	session: SessionResponse;
	messages: MessageResponse[];
}

/** Model info */
export interface ModelInfo {
	id: string;
	display_name: string;
	provider: string;
	context_window: number;
	max_output: number;
	supports_thinking: boolean;
	supports_tools: boolean;
}

/** Models response */
export interface ModelsResponse {
	models: ModelInfo[];
	default_model: string;
}

/** File tree entry */
export interface TreeEntry {
	name: string;
	path: string;
	is_dir: boolean;
	children?: TreeEntry[];
}

/** SSE stream event types */
export type StreamEvent =
	| { type: 'text_delta'; delta: string }
	| { type: 'thinking_delta'; thinking: string }
	| { type: 'tool_call_start'; id: string; name: string }
	| { type: 'tool_call_complete'; id: string; name: string; arguments: Record<string, unknown> }
	| { type: 'tool_executing'; id: string; name: string }
	| { type: 'tool_output_delta'; id: string; delta: string }
	| { type: 'tool_result'; id: string; output: string; is_error: boolean }
	| { type: 'plan_update'; items: PlanItem[] }
	| { type: 'mode_change'; mode: string; reason?: string }
	| { type: 'plan_complete'; tool_call_id: string; title: string; task_count: number }
	| { type: 'usage'; prompt_tokens: number; completion_tokens: number }
	| { type: 'title_update'; title: string }
	| { type: 'finish'; session_id: string }
	| { type: 'error'; error: string };

// ============================================================================
// API Client
// ============================================================================

// Store for current user ID (set from auth session)
let currentUserId: string | null = null;

export function setCurrentUserId(userId: string | null) {
	currentUserId = userId;
}

export function getCurrentUserId(): string | null {
	return currentUserId;
}

class ApiError extends Error {
	constructor(
		public status: number,
		message: string
	) {
		super(message);
		this.name = 'ApiError';
	}
}

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
	// Build headers with optional X-User-Id for multi-tenant auth
	const headers: Record<string, string> = {
		'Content-Type': 'application/json',
		...(options.headers as Record<string, string>)
	};

	if (currentUserId) {
		headers['X-User-Id'] = currentUserId;
	}

	const response = await fetch(`${API_BASE}${path}`, {
		...options,
		headers
	});

	if (!response.ok) {
		const error = await response.json().catch(() => ({ error: 'Unknown error' }));
		throw new ApiError(response.status, error.error || error.message || 'Request failed');
	}

	// Handle empty responses (204 No Content, etc.)
	const contentLength = response.headers.get('content-length');
	if (response.status === 204 || contentLength === '0') {
		return undefined as T;
	}

	return response.json();
}

export const apiClient = {
	// Sessions
	getSessions: () => request<SessionResponse[]>('/sessions'),

	getSession: (id: string) => request<SessionWithMessagesResponse>(`/sessions/${id}`),

	getDirectories: () => request<string[]>('/sessions/directories'),

	browseDirectories: (path?: string) =>
		request<{
			current: string;
			parent: string | null;
			directories: { name: string; path: string }[];
		}>(`/files/browse${path ? `?path=${encodeURIComponent(path)}` : ''}`),

	createSession: (title?: string, workingDir?: string) =>
		request<SessionResponse>('/sessions', {
			method: 'POST',
			body: JSON.stringify({ title, working_dir: workingDir })
		}),

	deleteSession: (id: string) =>
		request<void>(`/sessions/${id}`, { method: 'DELETE' }),

	updateSession: (id: string, data: { title?: string }) =>
		request<SessionResponse>(`/sessions/${id}`, {
			method: 'PATCH',
			body: JSON.stringify(data)
		}),

	pinchSession: (id: string, preservationHints?: string, direction?: string) =>
		request<{
			session: SessionResponse;
			summary: string;
			key_decisions: string[];
			pending_tasks: string[];
		}>(`/sessions/${id}/pinch`, {
			method: 'POST',
			body: JSON.stringify({
				preservation_hints: preservationHints,
				direction
			})
		}),

	getSessionState: (id: string) =>
		request<{
			id: string;
			agent_state: string;
			started_at: string | null;
			last_event_at: string | null;
		}>(`/sessions/${id}/state`),

	// Models
	getModels: () => request<ModelsResponse>('/models'),

	// Files
	getFile: (path: string) =>
		request<{ path: string; content: string; size: number }>(`/files?path=${encodeURIComponent(path)}`),

	writeFile: (path: string, content: string) =>
		request<{ path: string; bytes_written: number }>(`/files?path=${encodeURIComponent(path)}`, {
			method: 'PUT',
			body: JSON.stringify({ content })
		}),

	getFileTree: (root?: string, depth = 3) =>
		request<{ root: string; entries: TreeEntry[] }>(
			`/files/tree?${root ? `root=${encodeURIComponent(root)}&` : ''}depth=${depth}`
		),

	// Tools
	executeTool: (toolName: string, params: Record<string, unknown>) =>
		request<{ output: string; is_error: boolean }>('/tools/execute', {
			method: 'POST',
			body: JSON.stringify({ tool_name: toolName, params })
		}),

	// Submit a tool result (for interactive tools like AskUserQuestion)
	// Returns void - use streamToolResult for streaming continuation
	submitToolResult: (sessionId: string, toolCallId: string, result: string) =>
		request<void>('/chat/tool-result', {
			method: 'POST',
			body: JSON.stringify({ session_id: sessionId, tool_call_id: toolCallId, result })
		}),

};

// Chat streaming
export interface ImageContent {
	type: 'image';
	source: {
		type: 'base64';
		media_type: string;
		data: string;
	};
}

export interface TextContent {
	type: 'text';
	text: string;
}

export type ContentBlock = TextContent | ImageContent;

interface ChatRequest {
	session_id?: string;
	message: string;
	content?: ContentBlock[]; // For multi-modal (text + images)
	model?: string;
	thinking_enabled?: boolean;
}

export interface PlanItem {
	content: string;
	completed?: boolean;
}

interface StreamCallbacks {
	onTextDelta: (delta: string) => void;
	onThinkingDelta: (thinking: string) => void;
	onToolCallStart: (id: string, name: string) => void;
	onToolCallComplete: (id: string, name: string, args: Record<string, unknown>) => void;
	onToolResult: (id: string, output: string, isError: boolean) => void;
	onToolOutputDelta: (id: string, delta: string) => void;
	onPlanUpdate: (items: PlanItem[]) => void;
	onModeChange: (mode: string, reason?: string) => void;
	onPlanComplete: (toolCallId: string, title: string, taskCount: number) => void;
	onUsage: (promptTokens: number, completionTokens: number) => void;
	onTitleUpdate: (title: string) => void;
	onFinish: (sessionId: string) => void;
	onError: (error: string) => void;
}

interface ToolResultRequest {
	session_id: string;
	tool_call_id: string;
	result: string;
}

export async function streamToolResult(
	request: ToolResultRequest,
	callbacks: StreamCallbacks,
	signal?: AbortSignal
) {
	// Build headers with optional X-User-Id for multi-tenant auth
	const headers: Record<string, string> = { 'Content-Type': 'application/json' };
	if (currentUserId) {
		headers['X-User-Id'] = currentUserId;
	}

	const response = await fetch(`${API_BASE}/chat/tool-result`, {
		method: 'POST',
		headers,
		body: JSON.stringify(request),
		signal
	});

	if (!response.ok) {
		const error = await response.json().catch(() => ({ error: 'Unknown error' }));
		callbacks.onError(error.error || 'Request failed');
		return;
	}

	const reader = response.body?.getReader();
	if (!reader) {
		callbacks.onError('No response body');
		return;
	}

	const decoder = new TextDecoder();
	let buffer = '';
	let lastActivity = Date.now();

	// Activity timeout check
	const timeoutInterval = setInterval(() => {
		if (Date.now() - lastActivity > STREAM_ACTIVITY_TIMEOUT) {
			clearInterval(timeoutInterval);
			reader.cancel().catch(() => {});
			callbacks.onError('Stream timeout: no data received for 30 seconds');
		}
	}, STREAM_CHECK_INTERVAL);

	try {
		while (true) {
			const { done, value } = await reader.read();
			if (done) break;

			lastActivity = Date.now();
			buffer += decoder.decode(value, { stream: true });
			const lines = buffer.split('\n');
			buffer = lines.pop() || '';

			for (const line of lines) {
				if (line.startsWith('data: ')) {
					const data = line.slice(6);
					if (data === '[DONE]') continue;

					try {
						const event = JSON.parse(data);
						handleEvent(event, callbacks);
					} catch (e) {
						console.warn('[SSE] Parse error:', data, e);
					}
				}
			}
		}
	} catch (err) {
		if (err instanceof Error && err.name === 'AbortError') {
			// User cancelled
		} else {
			callbacks.onError(err instanceof Error ? err.message : 'Stream error');
		}
	} finally {
		clearInterval(timeoutInterval);
	}
}

export async function streamChat(
	request: ChatRequest,
	callbacks: StreamCallbacks,
	signal?: AbortSignal
) {
	// Build headers with optional X-User-Id for multi-tenant auth
	const headers: Record<string, string> = { 'Content-Type': 'application/json' };
	if (currentUserId) {
		headers['X-User-Id'] = currentUserId;
	}

	const response = await fetch(`${API_BASE}/chat`, {
		method: 'POST',
		headers,
		body: JSON.stringify(request),
		signal
	});

	if (!response.ok) {
		const error = await response.json().catch(() => ({ error: 'Unknown error' }));
		callbacks.onError(error.error || 'Request failed');
		return;
	}

	const reader = response.body?.getReader();
	if (!reader) {
		callbacks.onError('No response body');
		return;
	}

	const decoder = new TextDecoder();
	let buffer = '';
	let lastActivity = Date.now();

	// Activity timeout check
	const timeoutInterval = setInterval(() => {
		if (Date.now() - lastActivity > STREAM_ACTIVITY_TIMEOUT) {
			clearInterval(timeoutInterval);
			reader.cancel().catch(() => {});
			callbacks.onError('Stream timeout: no data received for 30 seconds');
		}
	}, STREAM_CHECK_INTERVAL);

	try {
		while (true) {
			const { done, value } = await reader.read();
			if (done) break;

			lastActivity = Date.now();
			buffer += decoder.decode(value, { stream: true });
			const lines = buffer.split('\n');
			buffer = lines.pop() || '';

			for (const line of lines) {
				if (line.startsWith('data: ')) {
					const data = line.slice(6);
					if (data === '[DONE]') continue;

					try {
						const event = JSON.parse(data);
						handleEvent(event, callbacks);
					} catch (e) {
						console.warn('[SSE] Parse error:', data, e);
					}
				}
			}
		}
	} catch (err) {
		if (err instanceof Error && err.name === 'AbortError') {
			// User cancelled
		} else {
			callbacks.onError(err instanceof Error ? err.message : 'Stream error');
		}
	} finally {
		clearInterval(timeoutInterval);
	}
}

function handleEvent(event: StreamEvent, callbacks: StreamCallbacks) {
	switch (event.type) {
		case 'text_delta':
			callbacks.onTextDelta(event.delta);
			break;
		case 'thinking_delta':
			callbacks.onThinkingDelta(event.thinking);
			break;
		case 'tool_call_start':
			callbacks.onToolCallStart(event.id, event.name);
			break;
		case 'tool_call_complete':
			callbacks.onToolCallComplete(event.id, event.name, event.arguments);
			break;
		case 'tool_executing':
			// Heartbeat - no-op (activity timeout updated by read loop)
			break;
		case 'tool_output_delta':
			callbacks.onToolOutputDelta(event.id, event.delta);
			break;
		case 'tool_result':
			callbacks.onToolResult(event.id, event.output, event.is_error);
			break;
		case 'plan_update':
			callbacks.onPlanUpdate(event.items);
			break;
		case 'mode_change':
			callbacks.onModeChange(event.mode, event.reason);
			break;
		case 'plan_complete':
			callbacks.onPlanComplete(event.tool_call_id, event.title, event.task_count);
			break;
		case 'usage':
			callbacks.onUsage(event.prompt_tokens, event.completion_tokens);
			break;
		case 'title_update':
			callbacks.onTitleUpdate(event.title);
			break;
		case 'finish':
			callbacks.onFinish(event.session_id);
			break;
		case 'error':
			callbacks.onError(event.error);
			break;
	}
}
