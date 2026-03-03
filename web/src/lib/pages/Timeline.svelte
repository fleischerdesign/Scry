<script lang="ts">
	import TimelineItem from "../components/TimelineItem.svelte";
	import { createTimelineQuery } from "../queries/timeline";
	import { router } from "../router.svelte";
	
	const timelineQuery = createTimelineQuery();

	$effect(() => {
		router.title = "Timeline";
	});
</script>

<div class="space-y-10 animate-in fade-in duration-500 w-full">
	<div class="flex items-center justify-between px-2">
		<button class="btn btn-primary" onclick={() => timelineQuery.refetch()} disabled={timelineQuery.isFetching}>
			{#if timelineQuery.isFetching}
				<span class="loading loading-spinner loading-sm"></span>
			{/if}
			Refresh
		</button>
	</div>

	<ul class="timeline timeline-vertical timeline-compact">
		{#each timelineQuery.data ?? [] as item, i}
			<TimelineItem
				{item}
				isFirst={i === 0}
				isLast={i === (timelineQuery.data?.length ?? 0) - 1}
			/>
		{/each}
	</ul>

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
