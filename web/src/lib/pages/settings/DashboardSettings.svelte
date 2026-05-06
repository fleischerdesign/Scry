<script lang="ts">
	import { api } from "../../api";
	import Card from "../../components/Card.svelte";
	import { createDashboardsQuery } from "../../queries/dashboards";
	import { router } from "../../router.svelte";

	let newDashboardName = $state("");

	const dashboardsQuery = createDashboardsQuery();

	$effect(() => {
		router.title = "Dashboards";
	});

	async function createDashboard() {
		if (!newDashboardName) return;
		await api.createDashboard(newDashboardName);
		newDashboardName = "";
		dashboardsQuery.refetch();
	}
</script>

<div class="space-y-6 animate-in slide-in-from-right-4 duration-300">
	<Card title="Create New Dashboard" subtitle="New Dashboard">
		<div class="flex gap-4 py-2">
			<input
				type="text"
				bind:value={newDashboardName}
				placeholder="Enter Dashboard Name..."
				class="input input-bordered font-mono text-sm flex-1"
			/>
			<button class="btn btn-primary btn-sm" onclick={createDashboard}>
				CREATE
			</button>
		</div>
	</Card>

	<div class="space-y-2">
		<h4
			class="text-xs font-bold tracking-wide opacity-70 px-2"
		>
			Existing Dashboards
		</h4>
		<div
			class="flex flex-col bg-base-200 rounded-3xl overflow-hidden border border-base-300 divide-y divide-base-300/50"
		>
			{#each dashboardsQuery.data ?? [] as dash}
				<div class="flex items-center justify-between p-4 px-6">
					<div class="flex items-center gap-4">
						<div class="w-2 h-2 rounded-full bg-secondary"></div>
						<span class="font-bold text-sm tracking-tight">{dash.name}</span>
						<span class="text-xs font-mono opacity-60"
							>/{dash.slug}</span
						>
					</div>
					<button
						class="btn btn-ghost btn-xs text-error opacity-70 hover:opacity-100"
						>DELETE</button
					>
				</div>
			{/each}
		</div>
	</div>
</div>
