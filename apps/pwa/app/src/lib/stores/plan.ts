import { writable } from 'svelte/store';

export interface PlanItem {
	id: string;
	content: string;
	completed: boolean;
}

interface PlanState {
	items: PlanItem[];
	isVisible: boolean;
}

const initialState: PlanState = {
	items: [],
	isVisible: false
};

export const planStore = writable<PlanState>(initialState);

let idCounter = 0;

export function addPlanItem(content: string) {
	const id = `plan-${++idCounter}`;
	planStore.update((s) => ({
		...s,
		items: [...s.items, { id, content, completed: false }],
		isVisible: true
	}));
	return id;
}

export function togglePlanItem(id: string) {
	planStore.update((s) => ({
		...s,
		items: s.items.map((item) =>
			item.id === id ? { ...item, completed: !item.completed } : item
		)
	}));
}

export function removePlanItem(id: string) {
	planStore.update((s) => ({
		...s,
		items: s.items.filter((item) => item.id !== id)
	}));
}

export function clearPlan() {
	planStore.update((s) => ({
		...s,
		items: [],
		isVisible: false
	}));
}

export function setPlanVisible(visible: boolean) {
	planStore.update((s) => ({ ...s, isVisible: visible }));
}

export function setPlanItems(items: Array<{ content: string; completed?: boolean }>) {
	planStore.update((s) => ({
		...s,
		items: items.map((item, i) => ({
			id: `plan-${++idCounter}`,
			content: item.content,
			completed: item.completed ?? false
		})),
		isVisible: items.length > 0
	}));
}
