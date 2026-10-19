import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vite';

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
	resolve: {
		// The web app's components and tokens, used as they are. Subpath imports
		// can't point outside this project, so this is a Vite alias (and a path
		// in tsconfig.json).
		alias: {
			$ui: fileURLToPath(new URL('../web/src/lib/ui', import.meta.url)),
			// The logo.
			$assets: fileURLToPath(new URL('../web/src/lib/assets', import.meta.url))
		},
		// The web app's components import these from outside this project, which
		// would look in web/node_modules. Resolve them from here instead.
		dedupe: ['svelte', '@carbon/icons', '@ibm/plex-sans', '@ibm/plex-mono']
	},
	server: {
		port: 5174,
		strictPort: true,
		// The web app's components live outside this project.
		fs: { allow: ['../web/src/lib/ui', '../web/src/lib/assets'] },
		// File change events may not cross the VM's shared folder; set VITE_POLL=1 to poll instead.
		watch: process.env.VITE_POLL ? { usePolling: true, interval: 300 } : undefined
	}
});
