import { apiClient } from '$api/client';

const PUSH_SUBSCRIBED_KEY = 'krusty-push-subscribed';

function urlBase64ToUint8Array(base64String: string): Uint8Array {
	const padding = '='.repeat((4 - (base64String.length % 4)) % 4);
	const base64 = (base64String + padding).replace(/-/g, '+').replace(/_/g, '/');
	const raw = atob(base64);
	const arr = new Uint8Array(raw.length);
	for (let i = 0; i < raw.length; i++) {
		arr[i] = raw.charCodeAt(i);
	}
	return arr;
}

function arrayBufferToBase64Url(buffer: ArrayBuffer): string {
	const bytes = new Uint8Array(buffer);
	let binary = '';
	for (const byte of bytes) {
		binary += String.fromCharCode(byte);
	}
	return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

export async function subscribeToPush(): Promise<boolean> {
	if (!('serviceWorker' in navigator) || !('PushManager' in window)) {
		console.warn('Push notifications not supported');
		return false;
	}

	const permission = await Notification.requestPermission();
	if (permission !== 'granted') {
		return false;
	}

	const registration = await navigator.serviceWorker.ready;

	const { public_key } = await apiClient.getVapidPublicKey();
	const applicationServerKey = urlBase64ToUint8Array(public_key);

	const subscription = await registration.pushManager.subscribe({
		userVisibleOnly: true,
		applicationServerKey
	});

	const p256dh = subscription.getKey('p256dh');
	const auth = subscription.getKey('auth');
	if (!p256dh || !auth) {
		throw new Error('Missing push subscription keys');
	}

	await apiClient.pushSubscribe({
		endpoint: subscription.endpoint,
		p256dh: arrayBufferToBase64Url(p256dh),
		auth: arrayBufferToBase64Url(auth)
	});

	try {
		localStorage.setItem(PUSH_SUBSCRIBED_KEY, 'true');
	} catch { /* ignore */ }

	return true;
}

export async function unsubscribeFromPush(): Promise<void> {
	if (!('serviceWorker' in navigator)) return;

	const registration = await navigator.serviceWorker.ready;
	const subscription = await registration.pushManager.getSubscription();

	if (subscription) {
		await apiClient.pushUnsubscribe({ endpoint: subscription.endpoint });
		await subscription.unsubscribe();
	}

	try {
		localStorage.removeItem(PUSH_SUBSCRIBED_KEY);
	} catch { /* ignore */ }
}

export function isPushSubscribed(): boolean {
	try {
		return localStorage.getItem(PUSH_SUBSCRIBED_KEY) === 'true';
	} catch {
		return false;
	}
}

export function isPushSupported(): boolean {
	return 'serviceWorker' in navigator && 'PushManager' in window && 'Notification' in window;
}
