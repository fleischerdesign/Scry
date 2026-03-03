import { auth } from "../auth.svelte";
import { ui } from "../ui.svelte";
import { queryClient } from "../query";
import { keys } from "../queries/keys";
import type { Event } from "../types/Event";

/**
 * Initializes the Server-Sent Events (SSE) connection.
 * Must be called during component initialization (e.g., inside App.svelte)
 * so that Svelte's $effect rune works correctly.
 */
export function useStreaming() {
    let eventSource: EventSource | null = null;

    $effect(() => {
        // Cleanup function for when the effect re-runs or component unmounts
        return () => {
            if (eventSource) {
                eventSource.close();
                eventSource = null;
            }
        };
    });

    $effect(() => {
        // Reactively reconnect if auth state changes
        if (!auth.isAuthenticated || !auth.apiKey) {
            if (eventSource) {
                eventSource.close();
                eventSource = null;
            }
            return;
        }

        if (eventSource) return; // Already connected

        const baseUrl = import.meta.env.VITE_API_URL || "http://127.0.0.1:3000/api/v1";
        const url = `${baseUrl.replace(/\/api\/v1\/?$/, "")}/api/v1/streams/live?api_key=${auth.apiKey}`;
        
        eventSource = new EventSource(url);
        
        eventSource.onmessage = (e) => {
            const event: Event = JSON.parse(e.data);
            
            // Inject the new event into ALL timeline caches (regardless of limit)
            const timelineQueries = queryClient.getQueriesData({ queryKey: keys.timeline.all() });
            
            timelineQueries.forEach(([queryKey, oldData]) => {
                queryClient.setQueryData(queryKey, (old: Event[] | undefined) => {
                    if (!old) return [event];
                    // Keep the same limit as the original query requested (extract from queryKey if possible, default 100)
                    const limit = (queryKey[2] as any)?.limit || 100;
                    return [event, ...old].slice(0, limit);
                });
            });
            
            // Global notification for real-time feedback
            const label = event.display_title || `New Event: ${event.category}`;
            ui.notify(label, event.display_subtitle || undefined, 'info');
        };

        eventSource.onerror = (e) => {
            console.error("SSE Connection Error", e);
            // Reconnection is handled automatically by the browser's EventSource
        };
    });
}
