export function getThemeCSS(name: string, fallback = "#888"): string {
	const el = document.documentElement;
	const val = getComputedStyle(el).getPropertyValue(name).trim();
	return val || fallback;
}
