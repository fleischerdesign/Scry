<script lang="ts">
    import { onMount } from 'svelte';
    import { api } from '../api';
    import { router } from '../router.svelte';
    import Card from '../components/Card.svelte';
    import TimelineItem from '../components/TimelineItem.svelte';

    const { ns, type, id } = router.getParams('/entity/:ns/:type/:id');
    
    let traits = $state<Record<string, any>>({});
    let relationships = $state<any[]>([]);
    let events = $state<any[]>([]);
    let loading = $state(true);

    async function loadData() {
        loading = true;
        try {
            const [entityData, fetchedEvents] = await Promise.all([
                api.getEntityTraits(ns, type, id),
                api.getEntityEvents(ns, type, id)
            ]);
            traits = entityData.traits || {};
            relationships = entityData.relationships || [];
            events = fetchedEvents;
        } catch (e) {
            console.error("Failed to load entity details", e);
        } finally {
            loading = false;
        }
    }

    onMount(loadData);

    const photoUrl = $derived(traits["scry.visual/photo"]);
</script>

<div class="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500 w-full max-w-4xl pb-20">
    <!-- Header ... (bleibt gleich) -->
    <div class="flex items-start gap-6 border-b border-base-300 pb-8">
        <button class="btn btn-ghost btn-sm btn-square mt-2" onclick={() => window.history.back()}>
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" /></svg>
        </button>

        {#if photoUrl}
            <div class="avatar">
                <div class="w-24 h-24 rounded-3xl shadow-2xl ring ring-primary/20">
                    <img src={photoUrl} alt={id} />
                </div>
            </div>
        {:else}
            <div class="w-24 h-24 rounded-3xl bg-base-300 flex items-center justify-center text-3xl font-black opacity-20">
                {id.charAt(0)}
            </div>
        {/if}

        <div class="flex-1">
            <div class="badge badge-primary badge-outline font-mono text-[9px] uppercase tracking-widest mb-2">{ns} / {type}</div>
            <h2 class="text-4xl font-black tracking-tighter italic uppercase text-base-content">{id}</h2>
            <div class="flex gap-4 mt-4 text-[10px] font-mono opacity-40 uppercase tracking-widest">
                <span>{events.length} Events Logged</span>
                <span>•</span>
                <span>{relationships.length} Relationships</span>
            </div>
        </div>
    </div>

    {#if loading}
        <div class="flex justify-center py-20">
            <span class="loading loading-ring loading-lg opacity-20"></span>
        </div>
    {:else}
        <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
            <!-- Left: Knowledge & Relationships -->
            <div class="space-y-8">
                <!-- Relationships -->
                {#if relationships.length > 0}
                    <div class="space-y-4">
                        <h3 class="text-xs font-black uppercase tracking-[0.3em] opacity-30 px-2">Knowledge Graph Connections</h3>
                        <div class="grid grid-cols-1 gap-2">
                            {#each relationships as rel}
                                {@const isSource = rel.source.id === id}
                                {@const target = isSource ? rel.target : rel.source}
                                <button 
                                    onclick={() => router.navigate(`/entity/${target.ns}/${target.typ}/${target.id}`)}
                                    class="w-full flex flex-col items-start p-4 bg-base-200 hover:bg-base-300 transition-all rounded-2xl border border-base-300/50 group"
                                >
                                    <span class="text-[8px] font-black uppercase opacity-30 mb-1">{rel.predicate.split('/').pop()}</span>
                                    <span class="font-bold text-xs group-hover:text-primary transition-colors">{target.id}</span>
                                    <span class="text-[9px] opacity-40 mt-1">{target.ns}/{target.typ}</span>
                                </button>
                            {/each}
                        </div>
                    </div>
                {/if}

                <!-- Traits -->
                <div class="space-y-4">
                    <h3 class="text-xs font-black uppercase tracking-[0.3em] opacity-30 px-2">Entity Traits</h3>
                    <div class="grid grid-cols-1 gap-3">
                        {#each Object.entries(traits) as [traitId, value]}
                            {#if traitId !== 'scry.visual/photo'}
                                <div class="bg-base-200 p-4 rounded-2xl border border-base-300/50">
                                    <p class="text-[9px] font-black uppercase opacity-30 mb-1">{traitId.split('/').pop()}</p>
                                    <p class="font-mono text-xs overflow-hidden text-ellipsis">{value}</p>
                                </div>
                            {/if}
                        {/each}
                    </div>
                </div>
            </div>

            <!-- Right: Event History -->
            <div class="md:col-span-2 space-y-6">
                <h3 class="text-xs font-black uppercase tracking-[0.3em] opacity-30 px-2">Activity Timeline</h3>
                <div class="bg-base-100/50 rounded-3xl p-2 border border-base-300/30">
                    <ul class="timeline timeline-vertical timeline-compact">
                        {#each events as item, i}
                            <TimelineItem 
                                {item} 
                                isFirst={i === 0} 
                                isLast={i === events.length - 1} 
                            />
                        {/each}
                    </ul>
                </div>
            </div>
        </div>
    {/if}
</div>
