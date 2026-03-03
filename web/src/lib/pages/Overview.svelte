<script lang="ts">
	import Stat from "../components/Stat.svelte";
	import { createTimelineQuery } from "../queries/timeline";
	import { createPluginsQuery } from "../queries/plugins";
	import { createDashboardsQuery } from "../queries/dashboards";
	import { router } from "../router.svelte";

	let { dailySummary = [] } = $props();

	const timelineQuery = createTimelineQuery(100);
	const pluginsQuery = createPluginsQuery();
	const dashboardsQuery = createDashboardsQuery();

	$effect(() => {
		router.title = "Overview";
	});

	function syncKernel() {
		timelineQuery.refetch();
		pluginsQuery.refetch();
		dashboardsQuery.refetch();
	}
</script>

<div class="space-y-12 animate-in fade-in duration-700 w-full pb-20">
	<!-- Daily Perspective Hero (Managed by Plugins) -->
	{#if dailySummary && dailySummary.length > 0}
		<div
			class="hero bg-base-100 rounded-3xl border border-base-300 shadow-sm overflow-hidden"
		>
			<div class="hero-content text-center py-12 px-8 flex-col w-full">
				<div class="max-w-3xl">
					<h2
						class="text-[10px] uppercase tracking-[0.4em] font-black text-primary opacity-50 mb-8"
					>
						Daily_Perspective
					</h2>
					<div class="space-y-6">
						{#each dailySummary as line}
							{#if line.toLowerCase().includes("no ") || line
									.toLowerCase()
									.includes("unavailable")}
								<p
									class="text-[10px] font-mono uppercase tracking-[0.2em] opacity-20 py-4 border-y border-base-content/5"
								>
									{line}
								</p>
							{:else}
								<p
									class="text-3xl font-light leading-tight tracking-tight text-base-content italic quote"
								>
									"{line}"
								</p>
							{/if}
						{/each}
					</div>
				</div>
			</div>
		</div>
	{/if}

	<!-- Core Platform Stats -->
	<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
		<Stat
			title="Core Intelligence"
			value={pluginsQuery.data?.length ?? 0}
			desc="Semantic plugins active"
			color="primary"
			trend="Stable"
		/>
		<Stat
			title="Event Density"
			value={timelineQuery.data?.length ?? 0}
			desc="Total life events logged"
			color="secondary"
			trend="+5%"
		/>
		<Stat
			title="Interface Nodes"
			value={dashboardsQuery.data?.length ?? 0}
			desc="Active UI layouts"
			color="accent"
		/>
	</div>

	<!-- Core Insights (In der Zukunft: Automatisch gewählte Graphen) -->
	<div class="alert bg-base-100 border border-base-300 shadow-sm py-8">
		<svg
			xmlns="http://www.w3.org/2000/svg"
			fill="none"
			viewBox="0 0 24 24"
			class="stroke-info shrink-0 w-6 h-6"
			><path
				stroke-linecap="round"
				stroke-linejoin="round"
				stroke-width="2"
				d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
			></path></svg
		>
		<div>
			<h3 class="font-bold text-sm uppercase tracking-wider">System AI Hint</h3>
			<div class="text-xs opacity-60">
				You listened to 12% more music than yesterday. Your average temperature
				was 1.2°C lower.
			</div>
		</div>
	</div>

	<div
		class="flex flex-col items-center gap-4 pt-10 border-t border-base-300 opacity-20 hover:opacity-100 transition-all duration-500"
	>
		<button
			class="btn btn-outline btn-xs font-mono tracking-widest px-8 hover:btn-primary uppercase"
			onclick={syncKernel}>Sync_Kernel</button
		>
	</div>
</div>

<style>
	.quote {
		background: linear-gradient(to right, hsl(var(--p)), hsl(var(--s)));
		-webkit-background-clip: text;
		-webkit-text-fill-color: transparent;
		display: inline-block;
	}
</style>
