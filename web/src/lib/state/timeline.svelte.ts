import { api } from "../api";
import { auth } from "../auth.svelte";
import { ui } from "../ui.svelte";
import type { Event } from "../types/Event";

class TimelineState {
    items = $state<Event[]>([]);
    loading = $state(false);
    private eventSource: EventSource | null = null;

    async load(limit = 100) {
        if (!auth.isAuthenticated) return;
        this.loading = true;
        try {
            this.items = await api.getTimeline(limit);
            this.setupSSE();
        } catch (e) {
            console.error("Failed to load timeline", e);
        } finally {
            this.loading = false;
        }
    }

    setupSSE() {
        if (this.eventSource) this.eventSource.close();
        if (!auth.apiKey) return;

        const url = `http://127.0.0.1:3000/api/v1/streams/live?api_key=${auth.apiKey}`;
        this.eventSource = new EventSource(url);
        
        this.eventSource.onmessage = (e) => {
            const event: Event = JSON.parse(e.data);
            this.items = [event, ...this.items].slice(0, 100);
            
            // Agnostische Benachrichtigung
            const label = event.display_title || `New Event: ${event.category}`;
            ui.notify(label, event.display_subtitle || undefined, 'info');
        };
    }

    cleanup() {
        if (this.eventSource) {
            this.eventSource.close();
            this.eventSource = null;
        }
    }
}

export const timeline = new TimelineState();
