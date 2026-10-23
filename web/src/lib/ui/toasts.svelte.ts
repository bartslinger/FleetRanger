// Toast notifications: short confirmations of a completed action, such as
// "Group saved.", shown by <Toasts> at the top right. Without actions they
// dismiss themselves, as Carbon allows; errors belong inline, near the cause.

export interface Toast {
	id: number;
	kind: 'success' | 'info';
	title: string;
}

/** How long a toast stays, in milliseconds. */
const DURATION = 5000;

let next = 0;

// Shared module state: only ever changed in the browser (after a form posts),
// never during server rendering, so requests can't see each other's toasts.
export const toasts: Toast[] = $state([]);

export function dismiss(id: number) {
	const index = toasts.findIndex((toast) => toast.id === id);
	if (index !== -1) toasts.splice(index, 1);
}

export function toast(title: string, kind: Toast['kind'] = 'success') {
	const id = next++;
	toasts.push({ id, kind, title });
	setTimeout(() => dismiss(id), DURATION);
}
