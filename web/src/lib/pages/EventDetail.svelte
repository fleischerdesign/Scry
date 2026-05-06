<script lang="ts">
	import { api } from "../api";
	import { router } from "../router.svelte";
	import { semanticService } from "../services/semantic.svelte";
	import Card from "../components/Card.svelte";
 import EntityLabel from "../components/EntityLabel.svelte";
 import Icon from "@iconify/svelte";
 import PageHeader from "../components/PageHeader.svelte";
 import PageLoading from "../components/PageLoading.svelte";
 import type { Event } from "../types/Event";

 const params = $derived(router.getParams("/event/:id"));
	let event = $state<Event | null>(null);
	let loading = $state(true);

	// Agnostically find all metrics in context
	const contextMetrics = $derived(event ? semanticService.getMetricsFromContext(event.context_info) : []);

	$effect(() => {
		router.title = "Event";
		if (params.id) {
			loadEvent(params.id);
		}
	});

	async function loadEvent(eventId: string) {
		loading = true;
		try {
			event = await api.getEvent(eventId);
		} catch (e) {
			console.error("Failed to load event", e);
		} finally {
			loading = false;
		}
	}
</script>

<div class="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500 w-full max-w-4xl pb-20">
 <PageHeader 
  title={event?.display_title || "Event Detail"}
  onBack={() => window.history.back()}
 >
  {#snippet image()}
   {#if event?.display_image}
    <div class="avatar">
     <div class="w-20 h-20 rounded-2xl shadow-xl ring-4 ring-base-100 overflow-hidden bg-base-300">
      <img src={event.display_image} alt={event.display_title || ''} class="object-cover w-full h-full" />
     </div>
    </div>
   {:else if event?.display_icon}
    <div class="w-20 h-20 rounded-2xl bg-base-200 flex items-center justify-center shadow-inner border border-base-300/50">
     <Icon icon={event.display_icon} class="w-8 h-8 opacity-60" />
    </div>
   {:else}
    <div class="w-20 h-20 rounded-2xl bg-base-200 flex items-center justify-center text-2xl font-bold opacity-60">
     {(event?.display_title || "E").charAt(0).toUpperCase()}
    </div>
   {/if}
  {/snippet}

  <div class="space-y-3">
   <div class="flex flex-wrap gap-2">
    <div class="badge badge-secondary badge-outline font-bold text-xs uppercase tracking-widest">Event</div>
    {#if event}
     <div class="badge badge-ghost bg-base-200 font-bold text-xs uppercase tracking-widest opacity-60">
      {event.category.split('.').pop()}
     </div>
    {/if}
   </div>
   
   {#if event?.display_subtitle}
    <p class="text-sm font-medium opacity-60 leading-relaxed max-w-xl">{event.display_subtitle}</p>
   {/if}

   <div class="flex gap-6 text-xs font-bold tracking-wide opacity-60">
    <div class="flex items-center gap-1.5">
     <Icon icon="lucide:clock" class="w-3 h-3" />
     <span>{event ? new Date(event.timestamp).toLocaleString() : 'Loading...'}</span>
    </div>
    {#if event?.source}
     <div class="flex items-center gap-1.5">
      <Icon icon="lucide:hard-drive" class="w-3 h-3" />
      <span>{event.source}</span>
     </div>
    {/if}
   </div>
  </div>
 </PageHeader>

	{#if loading}
		<PageLoading />
	{:else if event}
		<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
			<!-- Main Content -->
			<div class="md:col-span-2 space-y-6">
				<Card title="Raw Payload" subtitle={event.category}>
					<pre
						class="bg-base-300 p-4 rounded-xl font-mono text-xs overflow-x-auto text-primary/80">
      {JSON.stringify(event.payload, null, 4)}
     </pre>
				</Card>

				{#if event.metadata}
					<Card title="Metadata" subtitle="Enrichment Info">
						<div class="grid grid-cols-1 gap-2">
							{#each Object.entries(event.metadata) as [k, v]}
								<div
									class="flex justify-between items-center bg-base-200 p-3 rounded-xl border border-base-300/50"
								>
									<span class="text-xs font-bold opacity-70"
										>{k}</span
									>
									<span class="font-mono text-xs">{v}</span>
								</div>
							{/each}
						</div>
					</Card>
				{/if}
			</div>

			<!-- Sidebar Info -->
			<div class="space-y-6">
				{#if contextMetrics.length > 0}
					<Card title="Semantic Context" subtitle="Enriched Metrics">
						<div class="grid grid-cols-1 gap-2">
							{#each contextMetrics as metric}
								<button 
									onclick={() => metric.source_id && router.navigate(`/event/${metric.source_id}`)}
									class="flex justify-between items-center p-3 bg-base-200 rounded-xl border border-base-300/50 text-left {metric.source_id ? 'hover:bg-base-300 cursor-pointer transition-all active:scale-[0.98]' : ''}"
								>
									<span class="text-xs font-bold opacity-60">{semanticService.getLabel(metric.key)}</span>
									<span class="text-xs font-bold tracking-tight text-primary">
										{semanticService.formatValue(metric.value?.value ?? metric.value, { semantic_type: metric.key, unit: metric.value?.unit })}
									</span>
								</button>
							{/each}
						</div>
					</Card>
				{/if}

				<Card title="Context" subtitle="Technical Details">
					<div class="space-y-4">
						<div>
							<p class="text-xs font-bold opacity-60 mb-1">
								Event UUID
							</p>
							<p class="font-mono text-xs opacity-60 truncate">
								{params.id}
							</p>
						</div>
						<div>
							<p class="text-xs font-bold opacity-60 mb-1">
								Timestamp
							</p>
							<p class="font-mono text-xs">
								{new Date(event.timestamp).toLocaleString()}
							</p>
						</div>
						<div>
							<p class="text-xs font-bold opacity-60 mb-1">
								Source Node
							</p>
							<div class="badge badge-outline font-mono text-xs uppercase">
								{event.source}
							</div>
						</div>
					</div>
				</Card>

				{#if event.entities && event.entities.length > 0}
					<Card title="Linked Entities" subtitle="Relationships">
						<div class="space-y-4">
							{#each event.entities as ent}
								<div class="p-3 bg-base-200 hover:bg-base-300 transition-all rounded-xl border border-base-300/50">
									<EntityLabel namespace={ent.namespace} typ={ent.typ} id={ent.id} />
								</div>
							{/each}
						</div>
					</Card>
				{/if}
			</div>
		</div>
	{/if}
</div>
