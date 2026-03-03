import { createQuery } from '@tanstack/svelte-query';
import { api } from '../api';
import { keys } from './keys';
import { auth } from '../auth.svelte';
import type { PluginStatus } from '../types/PluginStatus';

export function createPluginsQuery() {
    return createQuery<PluginStatus[], Error>(() => ({
        queryKey: keys.plugins.all(),
        queryFn: () => api.getPlugins(),
        enabled: auth.isAuthenticated,
    }));
}
