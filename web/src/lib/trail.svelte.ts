import { afterNavigate, beforeNavigate } from '$app/navigation';

/**
 * The path-based breadcrumb: the pages the user went through to get here,
 * since the last starting point (the side navigation or the product name).
 *
 * Each step remembers its place in the browser's history, and following it
 * goes back there through the history, so the page is as the user left it
 * (filters, page, scroll position). Consecutive pages of the same route (a
 * list paged or filtered) are one step: the last of them. Loops stay, as the
 * trail is the user's route. Each history entry's trail is kept in
 * sessionStorage, so Back, Forward and reloading restore it.
 */

export interface Step {
	/** SvelteKit's position of the step's history entry. */
	index: number;
	href: string;
	label: string;
}

const STORAGE = 'fleetranger:trails';
/** Trails kept for this many history entries back. */
const KEEP = 500;

/** SvelteKit's position of the current history entry (read only). */
function position(): number | undefined {
	const metadata = history.state?.['sveltekit:metadata'] as { historyIndex?: number } | undefined;
	return metadata?.historyIndex;
}

let trails: Record<number, Step[]> = {};
let steps: Step[] = $state([]);
/** The current page's title, set by its page header. */
let title = '';
/** The page being left: the step it becomes on the next page. */
let current: { index: number; href: string; path: string; label: string; trail: Step[] } | null =
	null;
let starting = false;

function save() {
	try {
		sessionStorage.setItem(STORAGE, JSON.stringify(trails));
	} catch {
		// Storage full or blocked: the trail lives on in memory.
	}
}

export const trail = {
	/** The steps before the current page, oldest first. */
	get steps() {
		return steps;
	},
	/** Called by the page header with the page's title. */
	set title(value: string) {
		title = value;
	},
	/** Goes back to a step through the browser's history. */
	go(step: Step) {
		const here = position();
		if (here !== undefined) history.go(step.index - here);
	}
};

/** The navigation about to happen begins a new trail, e.g. from a link in
 * a header panel, which is global as the side navigation is. */
export function startTrail() {
	starting = true;
}

/** Marks a link as a starting point: following it begins a new trail. */
export function startsTrail(node: HTMLElement) {
	node.addEventListener('click', startTrail);
	return { destroy: () => node.removeEventListener('click', startTrail) };
}

/** Follows the navigation; call once, while the root layout initialises. */
export function trackTrail() {
	try {
		trails = JSON.parse(sessionStorage.getItem(STORAGE) ?? '{}');
	} catch {
		trails = {};
	}

	beforeNavigate(() => {
		if (current) current.label = title;
	});

	afterNavigate((navigation) => {
		const index = position();
		const to = navigation.to?.url;
		if (index === undefined || !to) return;

		let next: Step[];
		if (navigation.type === 'popstate' || navigation.type === 'enter') {
			next = trails[index] ?? [];
		} else if (starting || !current) {
			next = [];
		} else if (index === current.index || to.pathname === current.path) {
			// The same entry replaced, or the same page again: the same step.
			next = current.trail;
		} else {
			next = [...current.trail, { index: current.index, href: current.href, label: current.label }];
		}
		starting = false;
		steps = next;

		trails[index] = next;
		for (const key of Object.keys(trails)) {
			if (Number(key) < index - KEEP) delete trails[Number(key)];
		}
		save();
		current = {
			index,
			href: to.pathname + to.search,
			path: to.pathname,
			label: title,
			trail: next
		};
	});
}
