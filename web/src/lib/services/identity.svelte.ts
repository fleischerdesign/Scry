import { api } from '../api';
import type { ApiEntity } from '../types/ApiEntity';

export interface EntityRef {
    namespace: string;
    typ: string;
    id: string;
}

class IdentityService {
    private cache = $state<Record<string, ApiEntity>>({});
    private pending = new Set<string>();
    private timer: any = null;

    /**
     * Resolves an entity reference to its display name.
     * If the entity is not in cache, it will be queued for batch loading.
     */
    resolve(ref: EntityRef): string {
        const key = `${ref.namespace}:${ref.typ}:${ref.id}`;
        
        if (this.cache[key]) {
            return this.cache[key].display_title;
        }

        // Queue for background loading if not already pending
        if (!this.pending.has(key)) {
            this.pending.add(key);
            this.scheduleBatchLoad();
        }

        // Return a shortened version of the ID while loading if it looks like a UUID
        if (ref.id.length > 30 && ref.id.includes('-')) {
            return `${ref.typ.toUpperCase()} (${ref.id.substring(0, 8)})`;
        }
        return ref.id;
    }

    /**
     * Returns the full entity object from cache if available.
     */
    get(ref: EntityRef): ApiEntity | undefined {
        const key = `${ref.namespace}:${ref.typ}:${ref.id}`;
        return this.cache[key];
    }

    private scheduleBatchLoad() {
        if (this.timer) clearTimeout(this.timer);
        this.timer = setTimeout(() => this.performBatchLoad(), 50);
    }

    private async performBatchLoad() {
        const currentBatch = Array.from(this.pending);
        this.pending.clear();

        if (currentBatch.length === 0) return;

        const refs = currentBatch.map(key => {
            const [namespace, typ, id] = key.split(':');
            return { namespace, typ, id };
        });

        try {
            const results = await api.resolveEntities(refs);
            for (const entity of results) {
                const key = `${entity.namespace}:${entity.typ}:${entity.id}`;
                this.cache[key] = entity;
            }
        } catch (e) {
            console.error('Failed to resolve entities batch', e);
        }
    }
}

export const identityService = new IdentityService();
