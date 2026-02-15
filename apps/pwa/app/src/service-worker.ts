/// <reference lib="WebWorker" />

import { build, files, version } from '$service-worker';

const sw = self as unknown as ServiceWorkerGlobalScope;
const CACHE = `krusty-pwa-${version}`;
const ASSETS = [...build, ...files];

sw.addEventListener('install', (event) => {
	event.waitUntil(
		(async () => {
			const cache = await caches.open(CACHE);
			await cache.addAll(ASSETS);
			await sw.skipWaiting();
		})()
	);
});

sw.addEventListener('activate', (event) => {
	event.waitUntil(
		(async () => {
			const keys = await caches.keys();
			await Promise.all(keys.filter((key) => key !== CACHE).map((key) => caches.delete(key)));
			await sw.clients.claim();
		})()
	);
});

sw.addEventListener('fetch', (event) => {
	const { request } = event;
	if (request.method !== 'GET') {
		return;
	}

	const url = new URL(request.url);
	// Never cache API calls.
	if (url.pathname.startsWith('/api')) {
		return;
	}

	event.respondWith(
		(async () => {
			const cached = await caches.match(request);
			if (cached) {
				return cached;
			}

			try {
				const response = await fetch(request);
				if (response.ok && (url.origin === sw.location.origin || ASSETS.includes(url.pathname))) {
					const cache = await caches.open(CACHE);
					void cache.put(request, response.clone());
				}
				return response;
			} catch {
				return cached ?? Response.error();
			}
		})()
	);
});
