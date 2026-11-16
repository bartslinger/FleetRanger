// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
	namespace App {
		// interface Error {}
		interface Locals {
			theme: import('#lib/theme.js').Theme;
			/** Null on pages shown without logging in, or before an organisation
			 * is chosen. */
			account: import('#lib/server/backend.js').Account | null;
			/** Who is logged in when `account` is null: in no organisation yet. */
			person: import('#lib/server/backend.js').Person | null;
		}
		// interface PageData {}
		interface PageState {
			/** The Equipment page's tab, switched without loading the page. */
			equipmentTab?: import('#lib/equipment.js').Tab;
			/** The session open over the Sessions cards, at its own URL without
			 * loading its page. */
			session?: string;
		}
		// interface Platform {}
	}
}

export {};
