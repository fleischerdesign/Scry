<script lang="ts">
	import TimelineItem from "../components/TimelineItem.svelte";
	import TimelineGroup from "../components/TimelineGroup.svelte";
	import TimeTravelNav from "../components/TimeTravelNav.svelte";
	import { createTimelineQuery } from "../queries/timeline";
	import { router } from "../router.svelte";
	import PageHeader from "../components/PageHeader.svelte";
	import Icon from "@iconify/svelte";
	import type { Event } from "../types/Event";
	
	const timelineQuery = createTimelineQuery();

	let selectedNamespace = $state<string | null>(null);

	// Dynamically extract namespaces from the data for the filter dropdown
	const availableNamespaces = $derived.by(() => {
		const ns = new Set<string>();
		(timelineQuery.data ?? []).forEach(ev => {
			const parts = ev.category.split('.');
			if (parts.length > 0) ns.add(parts[0]);
		});
		return Array.from(ns).sort();
	});

	// Derived grouped events: Array of [dateString, Event[]]
	const groupedEvents = $derived.by(() => {
		const groups: Record<string, Event[]> = {};
		const filtered = (timelineQuery.data ?? []).filter(ev => {
			if (!selectedNamespace) return true;
			return ev.category.startsWith(selectedNamespace + '.');
		});

		filtered.forEach(ev => {
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

<div class="space-y-8 animate-in fade-in duration-500 w-full">
	<PageHeader 
		title="Timeline" 
		subtitle="A chronological view of all events across your connected plugins."
	>
		{#snippet actions()}
			<div class="flex items-center gap-2">
				<!-- Namespace Filter Dropdown -->
				<div class="dropdown dropdown-end">
					<div 
						tabindex="0" 
						role="button" 
						class="btn btn-sm btn-ghost gap-2 border border-base-300 rounded-xl font-bold {selectedNamespace ? 'text-primary border-primary/30 bg-primary/5' : 'opacity-60 hover:opacity-100'} transition-all"
					>
						<Icon icon="lucide:filter" class="w-4 h-4" />
						{selectedNamespace ? selectedNamespace.toUpperCase() : 'Filter'}
					</div>
					<ul tabindex="0" class="dropdown-content z-[20] menu p-2 shadow-2xl bg-base-100 border border-base-300 rounded-2xl w-52 mt-2">
						<li>
							<button 
								class="flex justify-between items-center {selectedNamespace === null ? 'active' : ''}"
								onclick={() => selectedNamespace = null}
							>
								<span class="font-bold text-xs tracking-wide">All Events</span>
								{#if selectedNamespace === null}
									<Icon icon="lucide:check" class="w-3 h-3" />
								{/if}
							</button>
						</li>
						<div class="divider my-1 opacity-10"></div>
						{#each availableNamespaces as ns}
							<li>
								<button 
									class="flex justify-between items-center {selectedNamespace === ns ? 'active' : ''}"
									onclick={() => selectedNamespace = ns}
								>
									<span class="font-bold text-xs tracking-wide">{ns}</span>
									{#if selectedNamespace === ns}
										<Icon icon="lucide:check" class="w-3 h-3" />
									{/if}
								</button>
							</li>
						{:else}
							<li class="disabled"><span class="text-xs opacity-70">No namespaces found</span></li>
						{/each}
					</ul>
				</div>

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
			</div>
		{/snippet}
	</PageHeader>

	<!-- Professional 2-Column Layout -->
	<div class="flex items-start">
		<!-- Sidebar Navigation Index (Left for better reading flow) -->
		<TimeTravelNav groups={groupedEvents} />

		<!-- Main Timeline Feed -->
		<div class="flex-1 space-y-4">
			{#each groupedEvents as [date, events]}
				<TimelineGroup {date} {events} />
			{:else}
				{#if !timelineQuery.isLoading}
					<div class="flex flex-col items-center justify-center py-32 opacity-60 border-2 border-dashed border-base-300 rounded-[3rem]">
						<Icon icon="lucide:ghost" class="w-12 h-12 mb-4" />
						<p class="text-xs tracking-wide font-bold">
							{selectedNamespace ? `No ${selectedNamespace} events found` : 'No events found'}
						</p>
					</div>
				{/if}
			{/each}

			{#if timelineQuery.isLoading}
				<div class="flex flex-col items-center justify-center py-20 opacity-70">
					<span class="loading loading-spinner loading-lg text-primary"></span>
				</div>
			{/if}
		</div>
	</div>
</div>
