// An app role's key as the backend accepts it (backend/src/oauth/apps.rs).

/** Why a key isn't valid, or undefined when it is. */
export function keyError(key: string): string | undefined {
	const trimmed = key.trim();
	if (!trimmed) return 'Enter the key.';
	if (!/^[a-z][a-z0-9_]{0,63}$/.test(trimmed)) {
		return 'Use lowercase letters, digits and underscores, starting with a letter.';
	}
	return undefined;
}
