<script lang="ts">
    import { api } from '../api';
    import { router } from '../router.svelte';
    import Card from '../components/Card.svelte';
    import TimelineItem from '../components/TimelineItem.svelte';
    import EntityLabel from '../components/EntityLabel.svelte';

    const params = $derived(router.getParams('/entity/:ns/:type/:id'));
    
    let traits = $state<Record<string, any>>({});
    let relationships = $state<any[]>([]);
    let events = $state<any[]>([]);
    let displayTitle = $state("");
    let displayImage = $state<string | null>(null);
    let loading = $state(true);

    async function loadData(ns: string, type: string, id: string) {
        loading = true;
        try {
            // Fetch entity details and related events in parallel
            const [entityData, fetchedEvents] = await Promise.all([
                api.getEntityTraits(ns, type, id),
                api.getEntityEvents(ns, type, id)
            ]);
            
            displayTitle = entityData.display_title || id;
            displayImage = entityData.display_image || null;
            
            traits = entityData.traits || {};
            relationships = entityData.relationships || [];
            events = fetchedEvents;
        } catch (e) {
            console.error("Failed to load entity details", e);
        } finally {
            loading = false;
        }
    }

    const groupedRelationships = $derived.by(() => {
        const groups: Record<string, any[]> = {};
        relationships.forEach(rel => {
            const label = rel.display_label || rel.predicate.split('/').pop() || rel.predicate;
            if (!groups[label]) groups[label] = [];
            groups[label].push(rel);
        });
        return groups;
    });

    $effect(() => {
        if (params.ns && params.type && params.id) {
            loadData(params.ns, params.type, params.id);
        }
    });

    $effect(() => {
        router.title = displayTitle.toUpperCase();
    });
</script>

<div class="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500 w-full max-w-4xl pb-20">
    <!-- Header -->
    <div class="flex items-start gap-6 border-b border-base-300 pb-8">
        <button class="btn btn-ghost btn-sm btn-square mt-2" onclick={() => window.history.back()}>
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" /></svg>
        </button>

        {#if displayImage}
            <div class="avatar">
                <div class="w-24 h-24 rounded-3xl shadow-2xl ring ring-primary/20 overflow-hidden bg-base-300">
                    <img src={displayImage} alt={displayTitle} class="object-cover w-full h-full" />
                </div>
            </div>
        {:else}
            <div class="w-24 h-24 rounded-3xl bg-base-300 flex items-center justify-center text-3xl font-black opacity-20">
                {displayTitle.charAt(0).toUpperCase()}
            </div>
        {/if}

        <div class="flex-1">
            <div class="badge badge-primary badge-outline font-mono text-[9px] uppercase tracking-widest mb-2">{params.ns} / {params.type}</div>
            <h2 class="text-4xl font-black tracking-tighter italic uppercase text-base-content">{displayTitle}</h2>
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
                    <div class="space-y-6">
                        <h3 class="text-xs font-black uppercase tracking-[0.3em] opacity-30 px-2 border-l-2 border-primary/20 ml-2">Knowledge Graph</h3>
                        
                        {#each Object.entries(groupedRelationships) as [predicate, rels]}
                            <div class="space-y-2">
                                <h4 class="text-[9px] font-bold uppercase opacity-20 px-2 tracking-widest">{predicate}</h4>
                                <div class="grid grid-cols-1 gap-2">
                                    {#each rels as rel}
                                        {@const isSource = rel.source.id === params.id}
                                        {@const target = isSource ? rel.target : rel.source}
                                        <button 
                                            onclick={() => router.navigate(`/entity/${target.ns}/${target.typ}/${target.id}`)}
                                            class="w-full flex items-center gap-3 p-3 bg-base-200 hover:bg-base-300 transition-all rounded-2xl border border-base-300/50 group text-left"
                                        >
                                            <div class="w-8 h-8 rounded-lg bg-base-300 flex items-center justify-center text-[10px] font-bold opacity-40 group-hover:text-primary group-hover:opacity-100 transition-all">
                                                {(target.display_title || target.id).charAt(0).toUpperCase()}
                                            </div>
                                            <div class="flex-1 min-w-0">
                                                <div class="font-bold text-xs truncate group-hover:text-primary transition-colors">{target.display_title || target.id}</div>
                                                <div class="text-[8px] opacity-30 uppercase font-mono tracking-tighter">{target.typ} {#if target.display_title}• {target.id}{/if}</div>
                                            </div>
                                        </button>
                                    {/each}
                                </div>
                            </div>
                        {/each}
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
                                    
                                    {#if typeof value === 'string' && value.split(':').length === 3 && value.includes('.')}
                                        <!-- Agnostic Link Detection (ns:type:id) -->
                                        {@const parts = value.split(':')}
                                        <div class="mt-1">
                                            <EntityLabel namespace={parts[0]} typ={parts[1]} id={parts[2]} />
                                        </div>
                                    {:else}
                                        <p class="font-mono text-xs overflow-hidden text-ellipsis">{value}</p>
                                    {/if}
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
