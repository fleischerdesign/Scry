import { api } from "../api";
import { auth } from "../auth.svelte";

class PluginState {
    items = $state<any[]>([]);
    loading = $state(false);

    async load() {
        if (!auth.isAuthenticated) return;
        this.loading = true;
        try {
            this.items = await api.getPlugins();
        } catch (e) {
            console.error("Failed to load plugins", e);
        } finally {
            this.loading = false;
        }
    }

    async poll(id: string) {
        try {
            await api.pollPlugin(id);
            await this.load();
        } catch (e) {
            console.error(`Failed to poll plugin ${id}`, e);
        }
    }

    get enrichers() {
        return this.items.filter(p => p.roles && p.roles.includes("enricher"));
    }
}

export const plugins = new PluginState();
