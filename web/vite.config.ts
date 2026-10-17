import adapter from '@sveltejs/adapter-node';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

// In development the browser talks to Vite only. Backend routes are proxied, so
// cookies and login behave as in production, where nginx routes by path.
const backendUrl = process.env.BACKEND_URL ?? 'http://127.0.0.1:8080';

export default defineConfig({
	plugins: [
		sveltekit({
			compilerOptions: {
				// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true
			},
			adapter: adapter()
		})
	],
	server: {
		proxy: {
			'/api': backendUrl,
			'/auth': backendUrl
		},
		// File change events may not cross the VM's shared folder; set VITE_POLL=1 to poll instead.
		watch: process.env.VITE_POLL ? { usePolling: true, interval: 300 } : undefined
	}
});
