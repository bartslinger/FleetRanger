import { afterNavigate } from '$app/navigation';

/**
 * Where a detail page was opened from, for Save and Cancel to go back to: the
 * list, a flight's page, and so on. The given fallback (the page's list) when
 * it was opened directly. Call while the component initialises.
 */
export function back(fallback: () => string): { readonly href: string } {
	let from: string | null = $state(null);
	afterNavigate(({ from: previous, to }) => {
		if (previous && previous.url.pathname !== to?.url.pathname) {
			from = previous.url.pathname + previous.url.search;
		}
	});
	return {
		get href() {
			return from ?? fallback();
		}
	};
}
