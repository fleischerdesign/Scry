<script lang="ts">
	import { router } from "../router.svelte";
	import { createPluginsQuery } from "../queries/plugins";
	import { createDashboardsQuery } from "../queries/dashboards";
	import PageHeader from "../components/PageHeader.svelte";
	import Icon from "@iconify/svelte";

	const pluginsQuery = createPluginsQuery();
	const dashboardsQuery = createDashboardsQuery();

	$effect(() => {
		router.title = "Settings";
	});
</script>

<div
	class="space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-500 w-full max-w-4xl"
>
	<PageHeader 
		title="Settings" 
		subtitle="Configure your digital identity, manage platform extensions, and customize visualization layouts."
	/>

	<!-- Home Assistant Style Vertical List -->
	<div
		class="flex flex-col bg-base-100 rounded-[2rem] overflow-hidden border border-base-300 divide-y divide-base-300/50 shadow-sm"
	>
		<button
			onclick={() => router.navigate("/settings/general")}
			class="flex items-center gap-5 p-6 hover:bg-base-200/50 transition-all group text-left w-full"
		>
			<div
				class="w-12 h-12 rounded-2xl bg-primary/10 flex items-center justify-center text-primary group-hover:scale-105 transition-transform"
			>
				<Icon icon="lucide:settings" class="w-6 h-6" />
			</div>
			<div class="flex-1">
				<h3 class="font-black text-base tracking-tight">
					General
				</h3>
				<p class="text-xs opacity-70 font-bold tracking-wide mt-1">
					Identity, Appearance & API Keys
				</p>
			</div>
			<Icon icon="lucide:chevron-right" class="h-4 w-4 opacity-60" />
		</button>

		<button
			onclick={() => router.navigate("/settings/plugins")}
			class="flex items-center gap-5 p-6 hover:bg-base-200/50 transition-all group text-left w-full"
		>
			<div
				class="w-12 h-12 rounded-2xl bg-secondary/10 flex items-center justify-center text-secondary group-hover:scale-105 transition-transform"
			>
				<Icon icon="lucide:puzzle" class="w-6 h-6" />
			</div>
			<div class="flex-1">
				<h3 class="font-black text-base tracking-tight">
					Extensions
				</h3>
				<p class="text-xs opacity-70 font-bold tracking-wide mt-1">
					Manage WASM Nodes, Sources & Enrichers
				</p>
			</div>
			<div class="flex items-center gap-3">
				<div class="badge badge-secondary badge-outline badge-sm opacity-70 font-mono text-xs">{pluginsQuery.data?.length ?? 0}</div>
				<Icon icon="lucide:chevron-right" class="h-4 w-4 opacity-60" />
			</div>
		</button>

		<button
			onclick={() => router.navigate("/settings/dashboards")}
			class="flex items-center gap-5 p-6 hover:bg-base-200/50 transition-all group text-left w-full"
		>
			<div
				class="w-12 h-12 rounded-2xl bg-warning/10 flex items-center justify-center text-warning group-hover:scale-105 transition-transform"
			>
				<Icon icon="lucide:layout" class="w-6 h-6" />
			</div>
			<div class="flex-1">
				<h3 class="font-black text-base tracking-tight">
					Dashboards
				</h3>
				<p class="text-xs opacity-70 font-bold tracking-wide mt-1">
					Layouts, Widgets & Visualization
				</p>
			</div>
			<div class="flex items-center gap-3">
				<div class="badge badge-warning badge-outline badge-sm opacity-70 font-mono text-xs">{dashboardsQuery.data?.length ?? 0}</div>
				<Icon icon="lucide:chevron-right" class="h-4 w-4 opacity-60" />
			</div>
		</button>
	</div>
</div>
