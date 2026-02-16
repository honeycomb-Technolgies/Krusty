import { apiClient } from '$api/client';

const PUSH_SUBSCRIBED_KEY = 'krusty-push-subscribed';

export interface PushState {
	supported: boolean;
	permission: NotificationPermission | 'unsupported';
	subscribed: boolean;
	endpoint: string | null;
}

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

function rememberSubscribed(value: boolean) {
	try {
		if (value) {
			localStorage.setItem(PUSH_SUBSCRIBED_KEY, 'true');
		} else {
			localStorage.removeItem(PUSH_SUBSCRIBED_KEY);
		}
	} catch {
		// ignore
	}
}

function hadPriorSubscriptionHint(): boolean {
	try {
		return localStorage.getItem(PUSH_SUBSCRIBED_KEY) === 'true';
	} catch {
		return false;
	}
}

async function getRegistration(): Promise<ServiceWorkerRegistration> {
	return navigator.serviceWorker.ready;
}

async function syncSubscriptionWithServer(subscription: PushSubscription): Promise<void> {
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
}

async function ensureSubscription(registration: ServiceWorkerRegistration): Promise<PushSubscription> {
	const existing = await registration.pushManager.getSubscription();
	if (existing) {
		return existing;
	}

	const { public_key } = await apiClient.getVapidPublicKey();
	const applicationServerKey = Uint8Array.from(urlBase64ToUint8Array(public_key));

	return registration.pushManager.subscribe({
		userVisibleOnly: true,
		applicationServerKey
	});
}

export async function subscribeToPush(): Promise<boolean> {
	if (!isPushSupported()) {
		console.warn('Push notifications not supported');
		return false;
	}

	const permission = await Notification.requestPermission();
	if (permission !== 'granted') {
		rememberSubscribed(false);
		return false;
	}

	const registration = await getRegistration();
	const subscription = await ensureSubscription(registration);
	await syncSubscriptionWithServer(subscription);
	rememberSubscribed(true);
	return true;
}

export async function unsubscribeFromPush(): Promise<void> {
	if (!isPushSupported()) return;

	const registration = await getRegistration();
	const subscription = await registration.pushManager.getSubscription();

	if (subscription) {
		try {
			await apiClient.pushUnsubscribe({ endpoint: subscription.endpoint });
		} catch (error) {
			console.warn('Failed to unsubscribe endpoint on server:', error);
		}
		await subscription.unsubscribe();
	}

	rememberSubscribed(false);
}

export async function getCurrentPushState(): Promise<PushState> {
	if (!isPushSupported()) {
		return {
			supported: false,
			permission: 'unsupported',
			subscribed: false,
			endpoint: null
		};
	}

	const registration = await getRegistration();
	const subscription = await registration.pushManager.getSubscription();

	return {
		supported: true,
		permission: Notification.permission,
		subscribed: !!subscription,
		endpoint: subscription?.endpoint ?? null
	};
}

export async function reconcilePushSubscription(): Promise<PushState> {
	if (!isPushSupported()) {
		return {
			supported: false,
			permission: 'unsupported',
			subscribed: false,
			endpoint: null
		};
	}

	const registration = await getRegistration();
	let subscription = await registration.pushManager.getSubscription();
	const permission = Notification.permission;

	if (subscription) {
		try {
			await syncSubscriptionWithServer(subscription);
		} catch (error) {
			console.warn('Failed to sync existing push subscription with server:', error);
		}
		rememberSubscribed(true);
		return {
			supported: true,
			permission,
			subscribed: true,
			endpoint: subscription.endpoint
		};
	}

	// Auto-heal after restarts/key drift: if browser permission is granted and the user
	// previously opted in, recreate the subscription and re-upsert server-side.
	if (permission === 'granted' && hadPriorSubscriptionHint()) {
		try {
			subscription = await ensureSubscription(registration);
			try {
				await syncSubscriptionWithServer(subscription);
			} catch (error) {
				console.warn('Push reconcile sync failed:', error);
			}
			rememberSubscribed(true);
			return {
				supported: true,
				permission,
				subscribed: true,
				endpoint: subscription.endpoint
			};
		} catch (error) {
			console.warn('Push reconcile auto-heal failed:', error);
		}
	}

	rememberSubscribed(false);
	return {
		supported: true,
		permission,
		subscribed: false,
		endpoint: null
	};
}

// Backward-compatible local hint; use getCurrentPushState()/reconcilePushSubscription() for truth.
export function isPushSubscribed(): boolean {
	return hadPriorSubscriptionHint();
}

export function isPushSupported(): boolean {
	if (typeof window === 'undefined' || typeof navigator === 'undefined') {
		return false;
	}
	return 'serviceWorker' in navigator && 'PushManager' in window && 'Notification' in window;
}
