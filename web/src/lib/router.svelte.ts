interface RouteConfig {
	path: string;
	label: string;
}

const ROUTES: RouteConfig[] = [
	{ path: "/overview", label: "Overview" },
	{ path: "/timeline", label: "Timeline" },
	{ path: "/explorer", label: "Explorer" },
	{ path: "/analytics", label: "Insights" },
	{ path: "/settings", label: "Settings" },
	{ path: "/settings/general", label: "General" },
	{ path: "/settings/plugins", label: "Plugins" },
	{ path: "/settings/enrichers", label: "Enrichers" },
	{ path: "/settings/dashboards", label: "Dashboards" },
	{ path: "/dashboard/:slug", label: "Dashboard" },
	{ path: "/event/:id", label: "Event" },
	{ path: "/entity/:ns/:type/:id", label: "Entity" },
];

class Router {
	path = $state(window.location.pathname || "/overview");
	
	// Registry für dynamische Titel (z.B. { "/event/123": "Meeting mit Max" })
	#dynamicTitles = $state<Record<string, string>>({});

	constructor() {
		window.addEventListener("popstate", () => {
			this.path = window.location.pathname || "/overview";
		});
		if (this.path === "/") this.path = "/overview";
	}

	navigate(newPath: string) {
		const target = newPath.startsWith("/") ? newPath : `/${newPath}`;
		window.history.pushState({}, "", target);
		this.path = target;
	}

	/**
	 * Ermöglicht es einer Seite, ihren Titel dynamisch zu setzen.
	 * Beispiel: router.title = "Event #42"
	 */
	set title(val: string) {
		this.#dynamicTitles[this.path] = val;
	}

	/**
	 * Gibt den Titel der aktuellen Seite zurück.
	 * Priorität: Dynamisch > Config > Prettified Path
	 */
	get title(): string {
		if (this.#dynamicTitles[this.path]) return this.#dynamicTitles[this.path];
		const route = ROUTES.find(r => this.match(r.path));
		return route?.label || this.#prettyName(this.path.split('/').pop() || "");
	}

	/**
	 * Berechnet Breadcrumbs. 
	 * Zerlegt den Pfad in Segmente und sucht für jedes Segment das Label.
	 */
	get breadcrumbs() {
		const parts = this.path.split('/').filter(Boolean);
		let currentPath = "";
		
		return parts.map((part, index) => {
			currentPath += `/${part}`;
			const isLast = index === parts.length - 1;
			
			// Für das letzte Element schauen wir auch in die dynamischen Titel
			const dynamic = isLast ? this.#dynamicTitles[this.path] : null;
			const config = ROUTES.find(r => this.match(r.path, currentPath));
			
			return {
				label: dynamic || config?.label || this.#prettyName(part),
				path: currentPath
			};
		});
	}

	#prettyName(str: string) {
		if (!str) return "";
		return str.charAt(0).toUpperCase() + str.slice(1).replace(/[-_]/g, ' ');
	}

	getParams(pattern: string): Record<string, string> {
		const pathParts = this.path.split('/');
		const patternParts = pattern.split('/');
		const params: Record<string, string> = {};
		patternParts.forEach((part, i) => {
			if (part.startsWith(':')) {
				params[part.slice(1)] = decodeURIComponent(pathParts[i] || "");
			}
		});
		return params;
	}

	match(pattern: string, pathToMatch: string = this.path): boolean {
		const pathParts = pathToMatch.split('/');
		const patternParts = pattern.split('/');
		if (pathParts.length !== patternParts.length) return false;
		return patternParts.every((part, i) => part.startsWith(':') || part === pathParts[i]);
	}
}

export const router = new Router();
