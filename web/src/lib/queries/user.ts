import { createQuery } from '@tanstack/svelte-query';
import { api } from '../api';
import { keys } from './keys';
import { auth } from '../auth.svelte';
import type { JsonValue } from '../types/serde_json/JsonValue';

type TraitsResult = { traits: Record<string, JsonValue>, relationships: any[] };

export function createSelfTraitsQuery() {
    return createQuery<TraitsResult, Error>(() => ({
        queryKey: keys.entity.self(),
        queryFn: () => api.getEntityTraits("scry.core", "user", "self"),
        enabled: auth.isAuthenticated,
    }));
}
