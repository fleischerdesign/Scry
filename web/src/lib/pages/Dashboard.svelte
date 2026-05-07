<script lang="ts">
	import Widget from "../components/Widget.svelte";
	import { api } from "../api";
	import { ui } from "../ui.svelte";
	import { router } from "../router.svelte";
	import { createDashboardsQuery } from "../queries/dashboards";
	import { createPluginsQuery } from "../queries/plugins";
	import PageLoading from "../components/PageLoading.svelte";
	import EmptyState from "../components/EmptyState.svelte";
	import type { ApiWidgetDefinition } from "../types/ApiWidgetDefinition";

	let isEditing = $state(false);
	let isCreating = $state(false);
	let isAddingWidget = $state(false);
	let newDashName = $state("");
	let deletingId = $state<string | null>(null);

	const dashboardsQuery = createDashboardsQuery();
	const pluginsQuery = createPluginsQuery();

	// Find active dashboard from URL via query data
	const activeDashboard = $derived.by(() => {
		const slug = router.path.split('/').pop();
		const items = dashboardsQuery.data ?? [];
		return items.find(d => d.slug === slug) || items[0] || null;
	});

	// Collect all suggested widgets from query data
	const widgetMarketplace = $derived.by(() => {
		const items = pluginsQuery.data ?? [];
		return items.flatMap((p) =>
			(p.suggested_widgets || []).map((w: ApiWidgetDefinition) => ({
				...w,
				pluginName: p.name,
				pluginId: p.id,
			})),
		);
	});

	$effect(() => {
		router.title = activeDashboard?.name || "Dashboard";
	});

	async function handleCreate() {
		if (!newDashName) return;
		try {
			await api.createDashboard(newDashName);
			ui.notify("Dashboard Created", newDashName, "success");
			newDashName = "";
			isCreating = false;
			dashboardsQuery.refetch();
		} catch (e) {
			console.error(e);
		}
	}

	async function removeWidget(widgetId: string) {
		if (!activeDashboard) return;
		deletingId = widgetId;
		try {
			await api.deleteWidget(activeDashboard.id, widgetId);
			ui.notify("Widget Removed", "", "info");
			dashboardsQuery.refetch();
		} catch (e) {
			console.error(e);
		} finally {
			deletingId = null;
		}
	}

	async function addSuggestedWidget(w: ApiWidgetDefinition & { pluginName: string }) {
		if (!activeDashboard) return;
		try {
			const config =
				typeof w.config_json === "string"
					? JSON.parse(w.config_json)
					: w.config_json;
			await api.addWidget(activeDashboard.id, {
				type: w.template,
				title: w.title,
				config: config,
			});
			ui.notify("Widget Added", w.title, "success");
			isAddingWidget = false;
			dashboardsQuery.refetch();
		} catch (e) {
			console.error("Failed to add widget", e);
		}
	}
</script>

<div class="space-y-10 animate-in fade-in duration-500 w-full pb-20">
	<div class="flex items-center justify-between pb-6">
		<div class="flex gap-2">
			<button
				class="btn btn-sm text-xs tracking-wide {isEditing
					? 'btn-primary'
					: 'btn-ghost opacity-70'}"
				onclick={() => (isEditing = !isEditing)}
			>
				{isEditing ? "Save Changes" : "Edit Layout"}
			</button>
			<button
				class="btn btn-sm btn-ghost opacity-70 text-xs"
				onclick={() => (isCreating = true)}>New Board</button
			>
		</div>
	</div>

	{#if isCreating}
		<div
			class="alert bg-base-100 border border-secondary/30 shadow-xl animate-in zoom-in-95 duration-200"
		>
			<div class="flex-1 flex items-center gap-4">
				<input
					type="text"
					bind:value={newDashName}
					placeholder="Dashboard Name..."
					class="input input-bordered input-sm font-mono flex-1"
				/>
				<button class="btn btn-secondary btn-sm" onclick={handleCreate}
					>Create</button
				>
				<button
					class="btn btn-ghost btn-sm"
					onclick={() => (isCreating = false)}>Cancel</button
				>
			</div>
		</div>
	{/if}
	{#if dashboardsQuery.isLoading}
		<PageLoading />
	{:else}

	<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
		{#if activeDashboard}
			{#each activeDashboard.widgets as widget (widget.id)}
				<div class="relative group h-full">
					<Widget {widget} />

					{#if isEditing}
						<button
							class="absolute -top-2 -right-2 btn btn-circle btn-error btn-xs shadow-lg animate-in zoom-in-50 z-10"
							onclick={() => removeWidget(widget.id)}
							disabled={deletingId === widget.id}
							aria-label="Remove widget"
						>
							<svg
								xmlns="http://www.w3.org/2000/svg"
								class="h-3 w-3"
								fill="none"
								viewBox="0 0 24 24"
								stroke="currentColor"
								><path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="3"
									d="M6 18L18 6M6 6l12 12"
								/></svg
							>
						</button>
					{/if}
				</div>
			{/each}
		{/if}

		<!-- Add Widget Placeholder (Only in Edit Mode) -->
		{#if isEditing}
			<button
				onclick={() => (isAddingWidget = true)}
				class="h-48 border-2 border-dashed border-primary/20 hover:border-primary/50 hover:bg-primary/5 transition-all rounded-3xl flex flex-col items-center justify-center gap-3 group"
			>
				<div
					class="w-10 h-10 rounded-2xl bg-primary/10 flex items-center justify-center text-primary group-hover:scale-110 transition-transform"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-6 w-6"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						><path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="3"
							d="M12 4v16m8-8H4"
						/></svg
					>
				</div>
				<span
					class="text-xs font-bold tracking-wide opacity-70 group-hover:opacity-100"
					>Add Widget</span
				>
			</button>
		{/if}
	</div>

	{#if !activeDashboard || (activeDashboard.widgets.length === 0 && !isEditing)}
		<EmptyState icon="lucide:layout-dashboard" title="Dashboard empty" description="Enter Edit Mode to add suggested widgets from your plugins." />
	{/if}

{/if}
</div>

<!-- Widget Marketplace Modal -->
{#if isAddingWidget}
	<div class="modal modal-open">
		<div
			class="modal-box max-w-2xl bg-base-100 border border-base-300 rounded-3xl p-0 overflow-hidden"
		>
			<div
				class="p-6 border-b border-base-200 flex justify-between items-center bg-base-200/50"
			>
				<div>
					<h3 class="font-black text-xl tracking-tight">Widget Marketplace</h3>
					<p class="text-xs opacity-70 tracking-wide font-bold">
						Suggested by active plugins
					</p>
				</div>
				<button
					class="btn btn-sm btn-circle btn-ghost"
					onclick={() => (isAddingWidget = false)}
					aria-label="Close">✕</button
				>
			</div>

			<div
				class="p-4 grid grid-cols-1 md:grid-cols-2 gap-4 max-h-[60vh] overflow-y-auto"
			>
				{#each widgetMarketplace as w}
					<button
						onclick={() => addSuggestedWidget(w)}
						class="flex flex-col items-start p-5 bg-base-200 hover:bg-base-300 transition-all rounded-2xl border border-transparent hover:border-primary/20 group text-left"
					>
						<div class="flex justify-between w-full items-start mb-3">
							<div
								class="badge badge-outline badge-xs font-mono opacity-70 uppercase"
							>
								{w.pluginName}
							</div>
							<div class="badge badge-primary badge-xs font-mono uppercase">
								{w.template}
							</div>
						</div>
						<span class="font-bold text-sm truncate w-full">{w.title}</span>
						<span class="text-xs opacity-70 mt-1 tracking-tighter"
							>Click to install recipe</span
						>
					</button>
				{:else}
					<EmptyState icon="lucide:puzzle" title="No suggested widgets found" description="More widgets coming soon from your plugins." />
				{/each}
			</div>

			<div class="p-4 bg-base-200/50 border-t border-base-200 text-center">
				<p class="text-xs font-bold opacity-60 tracking-wide">
					More widgets coming soon from your plugins
				</p>
			</div>
		</div>
		<button class="modal-backdrop" onclick={() => (isAddingWidget = false)}
			>close</button
		>
	</div>
{/if}
