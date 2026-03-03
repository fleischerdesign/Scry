import { createQuery } from '@tanstack/svelte-query';
import { api } from '../api';
import { keys } from './keys';
import { auth } from '../auth.svelte';
import type { Dashboard } from '../types/Dashboard';

export function createDashboardsQuery() {
    return createQuery<Dashboard[], Error>(() => ({
        queryKey: keys.dashboards.all(),
        queryFn: () => api.getDashboards(),
        enabled: auth.isAuthenticated,
    }));
}
