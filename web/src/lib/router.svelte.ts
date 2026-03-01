// Scry Minimal Router - Svelte 5 Rune Edition (History API)
class Router {
    path = $state(window.location.pathname || "/overview");

    constructor() {
        // Reagiere auf Zurück/Vorwärts im Browser
        window.addEventListener("popstate", () => {
            this.path = window.location.pathname || "/overview";
        });

        // Initialer Pfad-Check
        if (this.path === "/") this.path = "/overview";
    }

    navigate(newPath: string) {
        const target = newPath.startsWith("/") ? newPath : `/${newPath}`;
        window.history.pushState({}, "", target);
        this.path = target;
    }

    // Hilfsfunktion um Pfad-Parameter zu extrahieren (z.B. /event/123 -> {id: '123'})
    getParams(pattern: string): Record<string, string> {
        const pathParts = this.path.split('/');
        const patternParts = pattern.split('/');
        const params: Record<string, string> = {};

        patternParts.forEach((part, i) => {
            if (part.startsWith(':')) {
                const key = part.slice(1);
                params[key] = decodeURIComponent(pathParts[i] || "");
            }
        });

        return params;
    }

    match(pattern: string): boolean {
        const pathParts = this.path.split('/');
        const patternParts = pattern.split('/');
        if (pathParts.length !== patternParts.length) return false;
        
        return patternParts.every((part, i) => {
            return part.startsWith(':') || part === pathParts[i];
        });
    }
}

export const router = new Router();
