<script lang="ts">
	import Stat from "../components/Stat.svelte";
	import { api } from "../api";
	import { createTimelineQuery } from "../queries/timeline";
	import { createPluginsQuery } from "../queries/plugins";
	import { createDashboardsQuery } from "../queries/dashboards";
	import { router } from "../router.svelte";
	import PageHeader from "../components/PageHeader.svelte";
	import Icon from "@iconify/svelte";
	import EntityLabel from "../components/EntityLabel.svelte";

	let { dailySummary = [] } = $props();

	const timelineQuery = createTimelineQuery(100);
	const pluginsQuery = createPluginsQuery();
	const dashboardsQuery = createDashboardsQuery();

	// Agnostic Identity State
	let userEntity = $state<any>(null);
	let loadingUser = $state(true);

	async function loadUserIdentity() {
		try {
			userEntity = await api.getEntityTraits("scry.core", "user", "self");
		} catch (e) {
			console.error("Failed to load user identity", e);
		} finally {
			loadingUser = false;
		}
	}

	const userName = $derived(userEntity?.traits?.["scry.core/name"] || "User");
	const userPhoto = $derived(userEntity?.traits?.["scry.visual/photo"] || userEntity?.traits?.["scry.core/avatar"]);
	
	// Agnostically find all status traits
	const statusTraits = $derived.by(() => {
		if (!userEntity?.traits) return [];
		return Object.entries(userEntity.traits)
			.filter(([key]) => key.startsWith("scry.status/"))
			.map(([key, value]) => ({ key, value }));
	});

	$effect(() => {
		router.title = "Overview";
		loadUserIdentity();
	});

	function syncKernel() {
		timelineQuery.refetch();
		pluginsQuery.refetch();
		dashboardsQuery.refetch();
	}
</script>

<div class="space-y-12 animate-in fade-in duration-700 w-full pb-20">
	<PageHeader 
		title="Overview" 
		subtitle="A snapshot of your data and platform status."
	>
		{#snippet actions()}
			<button
				class="btn btn-sm btn-ghost gap-2 font-bold opacity-60 hover:opacity-100 transition-all border border-base-300 rounded-xl"
				onclick={syncKernel}
			>
				<Icon icon="lucide:refresh-cw" class="w-4 h-4" />
				Refresh
			</button>
		{/snippet}
	</PageHeader>

	<!-- Agnostic Identity Hero (Full Width) -->
	<div class="flex flex-col md:flex-row items-center gap-10 bg-base-100 border border-base-300 rounded-3xl p-10 shadow-sm relative overflow-hidden group w-full">
		<!-- Visual Ambient Background -->
		<div class="absolute top-0 right-0 w-96 h-96 bg-primary/5 rounded-full blur-[120px] -mr-48 -mt-48"></div>
		
		<!-- Avatar Section -->
		<div class="relative shrink-0">
			<div class="avatar">
				<div class="w-32 h-32 rounded-3xl shadow-2xl ring-8 ring-base-200 group-hover:ring-primary/10 transition-all duration-500 overflow-hidden bg-base-300">
					{#if userPhoto}
						<img src={userPhoto} alt={userName} class="object-cover" />
					{:else}
						<div class="w-full h-full flex items-center justify-center text-4xl font-bold opacity-60">
							{userName.charAt(0).toUpperCase()}
						</div>
					{/if}
				</div>
			</div>
			<!-- Online Indicator -->
			<div class="absolute -bottom-2 -right-2 w-8 h-8 rounded-full bg-success border-4 border-base-100 flex items-center justify-center shadow-lg">
				<div class="w-2 h-2 rounded-full bg-white animate-ping"></div>
			</div>
		</div>

		<!-- Info Section -->
		<div class="flex-1 text-center md:text-left space-y-4 relative z-10">
			<div>
				<h2 class="text-3xl font-black tracking-tight">{userName}</h2>
			</div>

			<!-- Dynamic Status Traits -->
			<div class="flex flex-wrap items-center justify-center md:justify-start gap-3 mt-4">
				{#each statusTraits as status}
					<div class="bg-base-200/50 border border-base-300/50 rounded-2xl px-4 py-2 flex items-center gap-3">
						<div class="w-2 h-2 rounded-full bg-primary animate-pulse"></div>
						<span class="text-xs font-bold tracking-wide opacity-70">{status.key.split('/').pop()?.replace('_', ' ')}:</span>
						
						{#if typeof status.value === 'string' && status.value.split(':').length === 3 && status.value.includes('.')}
							{@const parts = status.value.split(':')}
							<EntityLabel namespace={parts[0]} typ={parts[1]} id={parts[2]} inline={true} />
						{:else}
							<span class="text-xs font-bold tracking-tight">{status.value}</span>
						{/if}
					</div>
				{/each}
				
				{#if statusTraits.length === 0}
					<div class="text-xs font-bold opacity-60 tracking-wide">No active status detected</div>
				{/if}
			</div>

			<!-- Digital Shadow Narration (Daily Perspective) -->
			{#if dailySummary.length > 0}
				<div class="pt-4 border-t border-base-300/50">
					<div class="space-y-2">
						{#each dailySummary as line}
							<p class="text-sm font-medium italic opacity-60 leading-relaxed quote">
								"{line}"
							</p>
						{/each}
					</div>
				</div>
			{/if}
		</div>
	</div>

	<!-- Core Platform Stats -->
	<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
		<Stat
			title="Plugins"
			value={pluginsQuery.data?.length ?? 0}
			desc="Active plugins"
			color="primary"
			trend="Stable"
		/>
		<Stat
			title="Events"
			value={timelineQuery.data?.length ?? 0}
			desc="Total events logged"
			color="secondary"
			trend="+5%"
		/>
		<Stat
			title="Dashboards"
			value={dashboardsQuery.data?.length ?? 0}
			desc="Active dashboards"
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
			<h3 class="font-black text-sm tracking-wider">Daily Insight</h3>
			<div class="text-xs opacity-60">
				You listened to 12% more music than yesterday. Your average temperature
				was 1.2°C lower.
			</div>
		</div>
	</div>
</div>

<style>
	.quote {
		background: linear-gradient(to right, hsl(var(--p)), hsl(var(--s)));
		background-clip: text;
		-webkit-background-clip: text;
		-webkit-text-fill-color: transparent;
		display: inline-block;
	}
</style>
