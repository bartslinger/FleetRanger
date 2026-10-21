// A toolbar's keyboard, as WAI-ARIA's toolbar pattern (which Carbon's text
// toolbar refers to): one tab stop for the whole bar, the arrow keys (and
// Home, End) between its controls, Tab back to the one last used.

/** Its controls: links, enabled buttons, and elements marked
 * `data-roving="true"` (e.g. a cut-off name with a tooltip); `"false"`
 * takes one out again. */
const SELECTOR = 'a[href], button:not([disabled]), [data-roving="true"]';

export function roving(node: HTMLElement) {
	let current = 0;

	const items = () =>
		[...node.querySelectorAll<HTMLElement>(SELECTOR)].filter(
			(item) => item.getAttribute('aria-disabled') !== 'true'
		);

	/** Only the current control in the tab order. */
	function apply() {
		const list = items();
		current = Math.max(0, Math.min(current, list.length - 1));
		list.forEach((item, index) => (item.tabIndex = index === current ? 0 : -1));
		for (const off of node.querySelectorAll('[data-roving="false"]'))
			off.removeAttribute('tabindex');
	}

	function onkeydown(event: KeyboardEvent) {
		const list = items();
		const index = list.indexOf(document.activeElement as HTMLElement);
		if (index < 0) return;
		const next = {
			ArrowLeft: index - 1,
			ArrowRight: index + 1,
			Home: 0,
			End: list.length - 1
		}[event.key];
		if (next === undefined) return;
		event.preventDefault();
		current = (next + list.length) % list.length;
		apply();
		list[current].focus();
	}

	function onfocusin(event: FocusEvent) {
		const index = items().indexOf(event.target as HTMLElement);
		if (index < 0) return;
		current = index;
		apply();
	}

	node.addEventListener('keydown', onkeydown);
	node.addEventListener('focusin', onfocusin);
	// Controls come and go (e.g. a menu item's button, a name cut off).
	const changes = new MutationObserver(apply);
	changes.observe(node, {
		childList: true,
		subtree: true,
		attributes: true,
		attributeFilter: ['data-roving', 'disabled', 'aria-disabled']
	});
	apply();

	return {
		destroy() {
			node.removeEventListener('keydown', onkeydown);
			node.removeEventListener('focusin', onfocusin);
			changes.disconnect();
		}
	};
}
