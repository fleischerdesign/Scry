// Centralized Query Key Factory to ensure type safety and prevent typos
export const keys = {
    all: ['scry'] as const,
    
    // Domain: Dashboards
    dashboards: {
        all: () => [...keys.all, 'dashboards'] as const,
        detail: (slug: string) => [...keys.dashboards.all(), slug] as const,
    },

    // Domain: Timeline
    timeline: {
        all: () => [...keys.all, 'timeline'] as const,
        list: (limit: number) => [...keys.timeline.all(), { limit }] as const,
    },

    // Domain: Plugins
    plugins: {
        all: () => [...keys.all, 'plugins'] as const,
        detail: (id: string) => [...keys.plugins.all(), id] as const,
    },

    // Domain: Entity Explorer
    entity: {
        all: () => [...keys.all, 'entities'] as const,
        self: () => [...keys.entity.all(), 'scry.core', 'user', 'self'] as const,
    }
};
