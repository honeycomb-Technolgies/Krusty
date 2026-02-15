<script lang="ts">
	import ListChecks from 'lucide-svelte/icons/list-checks';
	import Check from 'lucide-svelte/icons/check';
	import X from 'lucide-svelte/icons/x';
	import ChevronUp from 'lucide-svelte/icons/chevron-up';
	import ChevronDown from 'lucide-svelte/icons/chevron-down';
	import { planStore, togglePlanItem, removePlanItem, clearPlan } from '$stores/plan';

	let isMinimized = $state(false);

	const completedCount = $derived($planStore.items.filter((i) => i.completed).length);
	const totalCount = $derived($planStore.items.length);
	const progress = $derived(totalCount > 0 ? (completedCount / totalCount) * 100 : 0);
</script>

{#if $planStore.isVisible && $planStore.items.length > 0}
	<div
		class="plan-tracker fixed right-4 top-20 z-50 w-72 rounded-xl border border-green-500/30 bg-card/95
			shadow-lg shadow-green-500/10 backdrop-blur-md"
	>
		<!-- Header -->
		<div
			class="flex items-center justify-between border-b border-green-500/20 px-3 py-2"
		>
			<div class="flex items-center gap-2">
				<ListChecks class="h-4 w-4 text-green-400" />
				<span class="text-sm font-medium text-green-400">Plan Mode</span>
				<span class="rounded-full bg-green-500/20 px-2 py-0.5 text-xs text-green-400">
					{completedCount}/{totalCount}
				</span>
			</div>
			<div class="flex items-center gap-1">
				<button
					onclick={() => (isMinimized = !isMinimized)}
					class="rounded p-1 text-muted-foreground hover:bg-muted hover:text-foreground"
				>
					{#if isMinimized}
						<ChevronDown class="h-3.5 w-3.5" />
					{:else}
						<ChevronUp class="h-3.5 w-3.5" />
					{/if}
				</button>
				<button
					onclick={clearPlan}
					class="rounded p-1 text-muted-foreground hover:bg-muted hover:text-foreground"
					title="Clear plan"
				>
					<X class="h-3.5 w-3.5" />
				</button>
			</div>
		</div>

		<!-- Progress bar -->
		<div class="h-1 w-full bg-muted">
			<div
				class="h-full bg-gradient-to-r from-green-500 to-emerald-400 transition-all duration-300"
				style="width: {progress}%"
			></div>
		</div>

		<!-- Items list -->
		{#if !isMinimized}
			<div class="max-h-64 overflow-y-auto p-2">
				<ul class="space-y-1">
					{#each $planStore.items as item (item.id)}
						<li class="group flex items-start gap-2 rounded-lg px-2 py-1.5 hover:bg-muted/50">
							<button
								onclick={() => togglePlanItem(item.id)}
								class="mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center rounded border
									transition-colors
									{item.completed
										? 'border-green-500 bg-green-500 text-white'
										: 'border-muted-foreground/50 hover:border-green-500'}"
							>
								{#if item.completed}
									<Check class="h-3 w-3" />
								{/if}
							</button>
							<span
								class="flex-1 text-sm leading-tight
									{item.completed ? 'text-muted-foreground line-through' : 'text-foreground'}"
							>
								{item.content}
							</span>
							<button
								onclick={() => removePlanItem(item.id)}
								class="shrink-0 rounded p-0.5 text-muted-foreground opacity-0 transition-opacity
									hover:bg-muted hover:text-foreground group-hover:opacity-100"
							>
								<X class="h-3 w-3" />
							</button>
						</li>
					{/each}
				</ul>
			</div>
		{/if}
	</div>
{/if}

<style>
	.plan-tracker {
		animation: slide-in 0.2s ease-out;
	}

	@keyframes slide-in {
		from {
			opacity: 0;
			transform: translateX(20px);
		}
		to {
			opacity: 1;
			transform: translateX(0);
		}
	}
</style>
