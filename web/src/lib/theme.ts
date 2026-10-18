// The user's choice of light or dark theme. It is kept in a cookie so the
// server renders the chosen theme straight away; without one the system
// setting applies.

export type Theme = 'system' | 'light' | 'dark';

export const THEME_COOKIE = 'theme';

export function parseTheme(value: string | undefined): Theme {
	return value === 'light' || value === 'dark' ? value : 'system';
}

/** Applies a choice in the browser and remembers it for a year. */
export function setTheme(theme: Theme) {
	const root = document.documentElement;
	if (theme === 'system') {
		delete root.dataset.theme;
		document.cookie = `${THEME_COOKIE}=; Path=/; Max-Age=0; SameSite=Lax`;
	} else {
		root.dataset.theme = theme;
		document.cookie = `${THEME_COOKIE}=${theme}; Path=/; Max-Age=31536000; SameSite=Lax`;
	}
}
