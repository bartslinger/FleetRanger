import { afterNavigate, goto } from '$app/navigation';

/**
 * Where a detail page was opened from, for Save and Cancel to go back to: the
 * list, a flight's page, and so on. The given fallback (the page's list) when
 * it was opened directly. Call while the component initialises.
 *
 * `go()` returns through the browser's history when the page was opened from
 * another of the app's pages, so the browser's Back button afterwards goes on
 * to the page before that (with its scroll position), not to this page again.
 * Opened directly, it replaces this page with the fallback.
 */
export function back(fallback: () => string): {
	readonly href: string;
	go: () => Promise<void>;
	onclick: (event: MouseEvent) => void;
} {
	let from: string | null = $state(null);
	afterNavigate(({ from: previous, to }) => {
		if (previous && previous.url.pathname !== to?.url.pathname) {
			from = previous.url.pathname + previous.url.search;
		}
	});

	function go() {
		if (from === null) {
			// eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved by the caller
			return goto(fallback(), { replaceState: true });
		}
		// The previous page loads its data again, so it shows the change.
		history.back();
		return Promise.resolve();
	}

	return {
		get href() {
			return from ?? fallback();
		},
		go,
		/** For a Cancel link to `href`: goes back as `go()` does; with a
		 * modifier key the link opens normally, e.g. in a new tab. */
		onclick(event: MouseEvent) {
			if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) {
				return;
			}
			event.preventDefault();
			void go();
		}
	};
}
