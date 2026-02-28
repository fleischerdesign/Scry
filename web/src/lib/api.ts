import { auth } from "./auth.svelte";

class ScryAPI {
	private baseUrl = "http://127.0.0.1:3000/api/v1";

	private async request(path: string, options: RequestInit = {}) {
		const headers = new Headers(options.headers || {});
		if (auth.apiKey) {
			headers.set("X-API-Key", auth.apiKey);
		}
		headers.set("Content-Type", "application/json");

		const url = path.startsWith("http") ? path : `${this.baseUrl}${path.startsWith("/") ? "" : "/"}${path}`;

		const response = await fetch(url, { ...options, headers });
		if (!response.ok) {
			const error = await response.json().catch(() => ({ error: "Unknown error" }));
			throw new Error(error.error || `API Error: ${response.statusText}`);
		}

		// Prüfe ob die Antwort leer ist (z.B. bei 200 OK ohne Body)
		const text = await response.text();
		return text ? JSON.parse(text) : {};
	}

	// Discovery
	getCatalog(): Promise<any> { return this.request("/discovery/catalog"); }
	search(q: string): Promise<any[]> { return this.request(`/discovery/search?q=${encodeURIComponent(q)}`); }

	// Data
	getData(path: string, limit = 50, offset = 0): Promise<any[]> {
		const separator = path.includes("?") ? "&" : "?";
		return this.request(`/data/${path}${separator}limit=${limit}&offset=${offset}`);
	}

	// Streams
	getTimeline(limit = 20): Promise<any[]> { return this.request(`/streams/timeline?limit=${limit}`); }
	getSummary(date?: string): Promise<string[]> { 
		return this.request(`/streams/summary${date ? `?date=${date}` : ""}`); 
	}

	// Analytics
	getAnalytics(subpath: string): Promise<any> {
		return this.request(`/analytics/${subpath}`);
	}
	getSemanticTop(type: string, limit = 10, days?: number): Promise<any[]> {
		const daysParam = days !== undefined ? `&days=${days}` : "";
		return this.request(`/analytics/semantic/top?semantic_type=${type}&limit=${limit}${daysParam}`);
	}
	getSemanticSeries(type: string, days = 7): Promise<any[]> {
		return this.request(`/analytics/semantic/series?semantic_type=${type}&days=${days}`);
	}

	// System & Dashboards
	getStatus(): Promise<any> { return this.request("/system/status"); }
	getPlugins(): Promise<any[]> { return this.request("/system/plugins"); }
	getProfile(): Promise<Record<string, string>> { return this.request("/system/profile"); }
	updateProfile(data: Record<string, string>): Promise<void> { 
		return this.request("/system/profile", { method: "POST", body: JSON.stringify(data) }); 
	}
	updatePluginConfig(id: string, data: Record<string, string>): Promise<void> { 
		return this.request(`/system/plugins/${id}/config`, { method: "POST", body: JSON.stringify(data) }); 
	}
	getPluginReport(pluginId: string, reportId: string): Promise<any> {
		return this.request(`/analytics/plugins/${pluginId}/reports/${reportId}`);
	}
	getDashboards(): Promise<any[]> { return this.request("/system/dashboards"); }
	createDashboard(name: string): Promise<void> {
		return this.request("/system/dashboards", { method: "POST", body: JSON.stringify({ name }) });
	}
	addWidget(dashboardId: string, widget: any): Promise<void> {
		return this.request(`/system/dashboards/${dashboardId}/widgets`, { method: "POST", body: JSON.stringify(widget) });
	}
	deleteWidget(dashboardId: string, widgetId: string): Promise<void> {
		return this.request(`/system/dashboards/${dashboardId}/widgets/${widgetId}`, { method: "DELETE" });
	}
	pollPlugin(id: string): Promise<{events_saved: number}> { 
		return this.request(`/system/plugins/${id}/poll`, { method: "POST" }); 
	}
}

export const api = new ScryAPI();
