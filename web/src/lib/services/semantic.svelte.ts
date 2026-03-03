import { api } from '../api';
import type { ApiDataField } from '../types/ApiDataField';

export type SemanticCategory = 'metric' | 'entity' | 'state' | 'unknown';

export interface SemanticInfo {
    category: SemanticCategory;
    subType: string;
    schemaOrg?: string;
    isPII: boolean;
}

class SemanticService {
    private cache = new Map<string, any>();

    /**
     * Parse a semantic type string (e.g., "metric.environment.temperature")
     * into its constituent parts.
     */
    parseType(semanticType: string | null | undefined): SemanticInfo {
        if (!semanticType) {
            return { category: 'unknown', subType: 'unknown', isPII: false };
        }

        const parts = semanticType.split('.');
        const category = parts[0] as SemanticCategory;
        const subType = parts.slice(1).join('.');

        return {
            category: ['metric', 'entity', 'state'].includes(category) ? category : 'unknown',
            subType,
            isPII: false // Default, will be updated by metadata if available
        };
    }

    /**
     * Resolve a semantic type to its schema.org mapping via the backend.
     */
    async resolveSchemaOrg(semanticType: string): Promise<string | undefined> {
        if (this.cache.has(semanticType)) return this.cache.get(semanticType);
        
        try {
            const mapping = await api.resolveSemantic(semanticType);
            this.cache.set(semanticType, mapping.schema_org_uri);
            return mapping.schema_org_uri;
        } catch (e) {
            return undefined;
        }
    }

    /**
     * Format a value based on its semantic metadata.
     */
    formatValue(value: any, field?: Partial<ApiDataField>): string {
        if (value === null || value === undefined) return '---';

        // Handle PII
        if (field?.privacy === 'pii') {
            return '•••••';
        }

        if (typeof value === 'number') {
            const formatted = value.toFixed(1);
            if (field?.unit) {
                return `${formatted} ${this.formatUnit(field.unit)}`;
            }
            return formatted;
        }

        return String(value);
    }

    /**
     * Standardize unit display names.
     */
    formatUnit(unit: string): string {
        const units: Record<string, string> = {
            'celsius': '°C',
            'fahrenheit': '°F',
            'percent': '%',
            'meters_per_second': 'm/s',
            'kilometers_per_hour': 'km/h',
            'beats_per_minute': 'BPM'
        };
        return units[unit.toLowerCase()] || unit;
    }

    /**
     * Get a human-readable label for a semantic type.
     */
    getLabel(semanticType: string): string {
        const info = this.parseType(semanticType);
        // Convert "environment.temperature" -> "Temperature"
        const lastPart = info.subType.split('.').pop() || info.subType;
        return lastPart.charAt(0).toUpperCase() + lastPart.slice(1).replace('_', ' ');
    }

    /**
     * Get an icon for a semantic type.
     */
    getIcon(semanticType: string): string {
        const icons: Record<string, string> = {
            'environment.temperature': 'lucide:thermometer',
            'music.energy_level': 'lucide:zap',
            'system.cpu': 'lucide:cpu',
            'system.memory': 'lucide:database'
        };
        
        const info = this.parseType(semanticType);
        return icons[info.subType] || 'lucide:activity';
    }

    /**
     * Helper to find all metrics in a context object.
     */
    getMetricsFromContext(context: any): Array<{ key: string, value: any, icon: string, source_id?: string }> {
        if (!context) return [];
        return Object.entries(context)
            .filter(([key]) => this.parseType(key).category === 'metric')
            .map(([key, val]: [string, any]) => ({
                key,
                value: val,
                icon: this.getIcon(key),
                source_id: val?.source_id
            }));
    }
}

export const semanticService = new SemanticService();
