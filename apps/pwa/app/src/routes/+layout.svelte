<script lang="ts">
	import '../app.css';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { MessageSquare, Terminal, Code2, Monitor, Menu } from 'lucide-svelte';
	import PlasmaBackground from '$components/chat/PlasmaBackground.svelte';
	import { goto } from '$app/navigation';
	import { validateWorkspace } from '$stores/workspace';
	import { loadSession } from '$stores/session';
	import { apiClient } from '$api/client';
	import { reconcilePushSubscription } from '$lib/push';

	interface NavItem {
		href: string;
		icon: typeof MessageSquare;
		label: string;
	}

	const navItems: NavItem[] = [
		{ href: '/app', icon: MessageSquare, label: 'Chat' },
		{ href: '/terminal', icon: Terminal, label: 'Terminal' },
		{ href: '/ide', icon: Code2, label: 'IDE' },
		{ href: '/workspace', icon: Monitor, label: 'Preview' },
		{ href: '/menu', icon: Menu, label: 'Menu' }
	];

	// iOS PWA viewport fix: set --vh variable to actual viewport height
	function setViewportHeight() {
		const viewportHeight = window.visualViewport?.height ?? window.innerHeight;
		const vh = viewportHeight * 0.01;
		document.documentElement.style.setProperty('--vh', `${vh}px`);
	}

	onMount(() => {
		const handleOrientationChange = () => {
			setTimeout(setViewportHeight, 100);
		};

		setViewportHeight();
		window.addEventListener('resize', setViewportHeight);
		window.visualViewport?.addEventListener('resize', setViewportHeight);
		// Also handle orientation change on iOS
		window.addEventListener('orientationchange', handleOrientationChange);

		void validateWorkspace(apiClient);
		if ('serviceWorker' in navigator) {
			void navigator.serviceWorker.register('/service-worker.js').then(() => {
				void reconcilePushSubscription().catch((error) => {
					console.warn('Push reconcile failed:', error);
				});
			});

			navigator.serviceWorker.addEventListener('message', (event) => {
				if (event.data?.type === 'notification-click' && event.data.session_id) {
					void loadSession(event.data.session_id);
					void goto('/app');
				}
			});
		}

		return () => {
			window.removeEventListener('resize', setViewportHeight);
			window.visualViewport?.removeEventListener('resize', setViewportHeight);
			window.removeEventListener('orientationchange', handleOrientationChange);
		};
	});

	let { children } = $props();

	const publicRoutes = ['/'];
	let isPublicRoute = $derived(publicRoutes.some((route) => $page.url.pathname === route));

	// Check if we're in the app section (show bottom nav)
	let isAppRoute = $derived(
		$page.url.pathname.startsWith('/app') ||
		$page.url.pathname.startsWith('/terminal') ||
		$page.url.pathname.startsWith('/ide') ||
		$page.url.pathname.startsWith('/workspace') ||
		$page.url.pathname.startsWith('/menu')
	);
</script>

{#if isPublicRoute}
	<!-- Public page (marketing pointer) -->
	{@render children()}
{:else}
	<!-- App pages -->
	<PlasmaBackground />
	<div class="app-container safe-top flex w-screen flex-col overflow-hidden">
		<main class="flex-1 overflow-hidden">
			{@render children()}
		</main>

		{#if isAppRoute}
			<nav class="safe-bottom flex h-16 shrink-0 items-center justify-around border-t border-border/50 bg-card/60 backdrop-blur-sm">
				{#each navItems as item}
					{@const isActive = $page.url.pathname === item.href ||
						(item.href !== '/app' && $page.url.pathname.startsWith(item.href))}
					<a
						href={item.href}
						class="flex flex-col items-center gap-1 px-4 py-2 transition-colors
							{isActive ? 'text-primary' : 'text-muted-foreground hover:text-foreground'}"
					>
						<item.icon class="h-5 w-5" />
						<span class="text-xs font-medium">{item.label}</span>
					</a>
				{/each}
			</nav>
		{/if}
	</div>
{/if}
