import { auth } from "./auth.svelte";
import { ui } from "./ui.svelte";
import type { Event } from "./types/Event";
import type { ApiNamespace } from "./types/ApiNamespace";
import type { ApiEntity } from "./types/ApiEntity";
import type { PluginStatus } from "./types/PluginStatus";
import type { Dashboard } from "./types/Dashboard";
import type { ApiReportData } from "./types/ApiReportData";
import type { CorrelationResult } from "./types/CorrelationResult";
import type { SemanticStats } from "./types/SemanticStats";
import type { JsonValue } from "./types/serde_json/JsonValue";

class ScryAPI {
	private baseUrl = import.meta.env.VITE_API_URL || "http://127.0.0.1:3000/api/v1";

	public async request<T>(path: string, options: RequestInit = {}): Promise<T> {
		const headers = new Headers(options.headers || {});
		if (auth.token) {
			headers.set("Authorization", `Bearer ${auth.token}`);
		}
		headers.set("Content-Type", "application/json");

		const url = path.startsWith("http") ? path : `${this.baseUrl}${path.startsWith("/") ? "" : "/"}${path}`;

		const response = await fetch(url, { ...options, headers });
		if (!response.ok) {
			const error = await response.json().catch(() => ({ error: "Unknown error" }));
			const errorMessage = error.error || `API Error: ${response.statusText}`;
			
			// Global error notification
			ui.notify("API Fehler", errorMessage, "error");
			
			throw new Error(errorMessage);
		}

		const text = await response.text();
		return text ? JSON.parse(text) : ({} as T);
	}

	// Discovery
	getCatalog(): Promise<Record<string, any[]>> { return this.request("/discovery/catalog"); }
	search(q: string): Promise<any[]> { return this.request(`/discovery/search?q=${encodeURIComponent(q)}`); }
	getNamespaces(): Promise<ApiNamespace[]> {
		return this.request("/discovery/entities");
	}
	getNamespaceTypes(namespace: string): Promise<string[]> {
		return this.request(`/discovery/entities/${namespace}`);
	}
	getEntities(namespace: string, type: string): Promise<ApiEntity[]> {
		return this.request(`/discovery/entities/${namespace}/${type}`);
	}
	getEntityTraits(namespace: string, type: string, id: string): Promise<{ traits: Record<string, JsonValue>, relationships: any[], display_title?: string, display_image?: string }> {
		return this.request(`/discovery/entities/${namespace}/${type}/${encodeURIComponent(id)}/traits`);
	}
	getEvent(id: string): Promise<Event> {
		return this.request(`/data/id/${id}`);
	}
	getEntityEvents(namespace: string, type: string, id: string): Promise<Event[]> {
		return this.request(`/data/entity/${namespace}/${type}/${encodeURIComponent(id)}`);
	}

	// Data
	getData(path: string, limit = 50, offset = 0): Promise<Event[]> {
		const separator = path.includes("?") ? "&" : "?";
		return this.request(`/data/${path}${separator}limit=${limit}&offset=${offset}`);
	}

	// Streams
	getTimeline(limit = 20, category?: string): Promise<Event[]> { 
		const catParam = category ? `&category=${category}` : "";
		return this.request(`/streams/timeline?limit=${limit}${catParam}`); 
	}
	getSummary(date?: string): Promise<string[]> { 
		return this.request(`/streams/summary${date ? `?date=${date}` : ""}`); 
	}

	// Analytics
	getAnalytics(subpath: string, options: RequestInit = {}): Promise<any> {
		return this.request(`/analytics/${subpath}`, options);
	}
	getSemanticTop(type: string, limit = 10, days?: number): Promise<any[]> {
		const daysParam = days !== undefined ? `&days=${days}` : "";
		return this.request(`/analytics/semantic/top?semantic_type=${type}&limit=${limit}${daysParam}`);
	}
	getSemanticSeries(type: string, days = 7, interval?: string): Promise<any[]> {
		const intervalParam = interval ? `&interval=${interval}` : "";
		return this.request(`/analytics/semantic/series?semantic_type=${type}&days=${days}${intervalParam}`);
	}
    correlateEvents(params: { base_semantic?: string, join_semantic?: string, base_category?: string, join_category?: string, limit?: number }): Promise<CorrelationResult[]> {
        const query = new URLSearchParams(params as any).toString();
        return this.request(`/analytics/correlations?${query}`);
    }
    getSemanticStats(params: { base_semantic: string, join_semantic: string, limit?: number }): Promise<SemanticStats> {
        const query = new URLSearchParams(params as any).toString();
        return this.request(`/analytics/stats?${query}`);
    }

	// System & Dashboards
	getStatus(): Promise<{ status: string, multi_tenant: boolean }> { return this.request("/system/status"); }
	getPlugins(): Promise<PluginStatus[]> { return this.request("/system/plugins"); }
	getProfile(): Promise<Record<string, JsonValue>> { return this.request("/system/profile"); }
	updateProfile(data: Record<string, JsonValue>): Promise<void> { 
		return this.request("/system/profile", { method: "POST", body: JSON.stringify(data) }); 
	}
	updatePluginConfig(id: string, data: Record<string, JsonValue>): Promise<void> { 
		return this.request(`/system/plugins/${id}/config`, { method: "POST", body: JSON.stringify(data) }); 
	}
	getPluginSecrets(id: string): Promise<Record<string, string>> { 
		return this.request(`/system/plugins/${id}/secrets`); 
	}
	getPluginAuthUrl(id: string): Promise<{ auth_url?: string; state?: string; error?: string }> { 
		return this.request(`/system/plugins/${id}/auth-url`); 
	}
	getPluginReport(pluginId: string, reportId: string): Promise<ApiReportData> {
		return this.request(`/system/plugins/${pluginId}/reports/${reportId}`);
	}
	getDashboards(): Promise<Dashboard[]> { return this.request("/system/dashboards"); }
	createDashboard(name: string): Promise<void> {
		return this.request("/system/dashboards", { method: "POST", body: JSON.stringify({ name }) });
	}
	addWidget(dashboardId: string, widget: { type: string, title?: string, config: JsonValue }): Promise<void> {
		return this.request(`/system/dashboards/${dashboardId}/widgets`, { method: "POST", body: JSON.stringify(widget) });
	}
	deleteWidget(dashboardId: string, widgetId: string): Promise<void> {
		return this.request(`/system/dashboards/${dashboardId}/widgets/${widgetId}`, { method: "DELETE" });
	}
	pollPlugin(id: string): Promise<{events_saved: number}> { 
		return this.request(`/system/plugins/${id}/poll`, { method: "POST" }); 
	}

	// Semantic
	resolveSemantic(type: string): Promise<{ scry_type: string, schema_org_uri?: string, description?: string }> {
		return this.request(`/semantic/resolve?type=${encodeURIComponent(type)}`);
	}
}

export const api = new ScryAPI();
