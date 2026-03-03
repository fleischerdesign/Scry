import { createQuery } from '@tanstack/svelte-query';
import { api } from '../api';
import { keys } from './keys';
import { auth } from '../auth.svelte';
import type { Event } from '../types/Event';

export function createTimelineQuery(limit: number = 100) {
    return createQuery<Event[], Error>(() => ({
        queryKey: keys.timeline.list(limit),
        queryFn: () => api.getTimeline(limit),
        enabled: auth.isAuthenticated,
    }));
}
