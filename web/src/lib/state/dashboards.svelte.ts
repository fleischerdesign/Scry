import { api } from "../api";
import { auth } from "../auth.svelte";
import { router } from "../router.svelte";

class DashboardState {
    items = $state<any[]>([]);
    loading = $state(false);

    async load() {
        if (!auth.isAuthenticated) return;
        this.loading = true;
        try {
            this.items = await api.getDashboards();
        } catch (e) {
            console.error("Failed to load dashboards", e);
        } finally {
            this.loading = false;
        }
    }

    get active() {
        const slug = router.path.split('/').pop();
        return this.items.find(d => d.slug === slug) || this.items[0] || null;
    }
}

export const dashboards = new DashboardState();
