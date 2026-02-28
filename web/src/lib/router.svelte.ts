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
}

export const router = new Router();
