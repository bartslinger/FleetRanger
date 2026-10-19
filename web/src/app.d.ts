// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
	namespace App {
		// interface Error {}
		interface Locals {
			theme: import('#lib/theme.js').Theme;
			/** Null only on pages shown without logging in. */
			account: import('#lib/server/backend.js').Account | null;
		}
		// interface PageData {}
		interface PageState {
			/** The Equipment page's tab, switched without loading the page. */
			equipmentTab?: import('#lib/equipment.js').Kind;
		}
		// interface Platform {}
	}
}

export {};
