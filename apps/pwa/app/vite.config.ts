import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
	build: {
		target: 'esnext'
	},
	optimizeDeps: {
		esbuildOptions: {
			target: 'esnext'
		}
	},
	server: {
		host: true,
		port: 5173,
		proxy: {
			'/api': {
				target: process.env.KRUSTY_SERVER_ORIGIN || 'http://localhost:3000',
				changeOrigin: true,
				secure: false
			},
			'/ws': {
				target: process.env.KRUSTY_SERVER_ORIGIN || 'http://localhost:3000',
				ws: true,
				secure: false
			}
		}
	}
});
