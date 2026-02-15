declare global {
	namespace App {
		interface Platform {
			env?: {
				VITE_API_BASE?: string;
			};
		}
	}
}

export {};
