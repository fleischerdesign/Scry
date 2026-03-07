<script lang="ts">
	import TimelineItem from "../components/TimelineItem.svelte";
	import TimelineGroup from "../components/TimelineGroup.svelte";
	import { createTimelineQuery } from "../queries/timeline";
	import { router } from "../router.svelte";
	import PageHeader from "../components/PageHeader.svelte";
	import Icon from "@iconify/svelte";
	import type { Event } from "../types/Event";
	
	const timelineQuery = createTimelineQuery();

	// Derived grouped events: Array of [dateString, Event[]]
	const groupedEvents = $derived.by(() => {
		const groups: Record<string, Event[]> = {};
		(timelineQuery.data ?? []).forEach(ev => {
			const date = new Date(ev.timestamp).toISOString().split('T')[0];
			if (!groups[date]) groups[date] = [];
			groups[date].push(ev);
		});
		return Object.entries(groups).sort((a, b) => b[0].localeCompare(a[0]));
	});

	$effect(() => {
		router.title = "Timeline";
	});
</script>

<div class="space-y-10 animate-in fade-in duration-500 w-full">
	<PageHeader 
		title="Timeline" 
		subtitle="A real-time, chronological stream of all ingested life events across your connected nodes."
	>
		{#snippet actions()}
			<button 
				class="btn btn-sm btn-ghost gap-2 border border-base-300 rounded-xl font-bold opacity-60 hover:opacity-100 transition-all" 
				onclick={() => timelineQuery.refetch()} 
				disabled={timelineQuery.isFetching}
			>
				{#if timelineQuery.isFetching}
					<span class="loading loading-spinner loading-xs"></span>
				{:else}
					<Icon icon="lucide:refresh-cw" class="w-4 h-4" />
				{/if}
				Refresh
			</button>
		{/snippet}
	</PageHeader>

	<div class="space-y-4">
		{#each groupedEvents as [date, events]}
			<TimelineGroup {date} {events} />
		{/each}
	</div>

	{#if timelineQuery.isLoading}
		<div class="flex flex-col items-center justify-center py-20 opacity-50">
			<span class="loading loading-spinner loading-lg text-primary"></span>
		</div>
	{:else if (timelineQuery.data ?? []).length === 0}
		<div
			class="flex flex-col items-center justify-center py-20 opacity-20 border-2 border-dashed border-base-300 rounded-3xl"
		>
			<p class="font-mono text-xs uppercase tracking-widest">
				No events in current epoch
			</p>
		</div>
	{/if}
</div>
