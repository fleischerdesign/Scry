<script lang="ts">
    import { api } from '../api';
    import { router } from '../router.svelte';
    import { semanticService } from '../services/semantic.svelte';
    import Card from '../components/Card.svelte';
    import TimelineItem from '../components/TimelineItem.svelte';
    import EntityLabel from '../components/EntityLabel.svelte';
    import Icon from "@iconify/svelte";
    import PageHeader from '../components/PageHeader.svelte';
    import EntityGraph from '../components/EntityGraph.svelte';

    const params = $derived(router.getParams('/entity/:ns/:type/:id'));
    
    let traits = $state<Record<string, any>>({});
    let relationships = $state<any[]>([]);
    let events = $state<any[]>([]);
    let displayTitle = $state("");
    let displaySubtitle = $state<string | null>(null);
    let displayImage = $state<string | null>(null);
    let displayIcon = $state<string | null>(null);
    let loading = $state(true);
    let viewMode = $state<"list" | "graph">("list");

    async function loadData(ns: string, type: string, id: string) {
        loading = true;
        try {
            // Fetch entity details and related events in parallel
            const [entityData, fetchedEvents] = await Promise.all([
                api.getEntityTraits(ns, type, id),
                api.getEntityEvents(ns, type, id)
            ]);
            
            displayTitle = entityData.display_title || id;
            displaySubtitle = entityData.display_subtitle || null;
            displayImage = entityData.display_image || null;
            displayIcon = entityData.display_icon || null;
            
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
    <PageHeader title={displayTitle}>
        {#snippet actions()}
            <button class="btn btn-ghost btn-sm btn-square rounded-xl border border-base-300" onclick={() => window.history.back()}>
                <Icon icon="lucide:arrow-left" class="w-4 h-4" />
            </button>
        {/snippet}

        <div class="flex items-start gap-6 pt-4">
            {#if displayImage}
                <div class="avatar">
                    <div class="w-24 h-24 rounded-3xl shadow-xl ring-4 ring-base-100 overflow-hidden bg-base-300">
                        <img src={displayImage} alt={displayTitle} class="object-cover w-full h-full" />
                    </div>
                </div>
            {:else if displayIcon}
                <div class="w-24 h-24 rounded-3xl bg-base-200 flex items-center justify-center shadow-inner border border-base-300/50">
                    <Icon icon={displayIcon} class="w-10 h-10 opacity-30" />
                </div>
            {:else}
                <div class="w-24 h-24 rounded-3xl bg-base-200 flex items-center justify-center text-3xl font-black opacity-30">
                    {displayTitle.charAt(0).toUpperCase()}
                </div>
            {/if}

            <div class="flex-1 space-y-3">
                <div class="flex flex-wrap gap-2">
                    <div class="badge badge-primary badge-outline font-bold text-[10px] uppercase tracking-wider">{params.ns}</div>
                    <div class="badge badge-ghost bg-base-200 font-bold text-[10px] uppercase tracking-wider opacity-60">{params.type}</div>
                </div>
                
                {#if displaySubtitle}
                    <p class="text-sm font-medium opacity-60 leading-relaxed max-w-xl">{displaySubtitle}</p>
                {/if}

                <div class="flex gap-6 text-[10px] font-black uppercase tracking-[0.1em] opacity-30">
                    <div class="flex items-center gap-1.5">
                        <Icon icon="lucide:activity" class="w-3 h-3" />
                        <span>{events.length} Events</span>
                    </div>
                    <div class="flex items-center gap-1.5">
                        <Icon icon="lucide:share-2" class="w-3 h-3" />
                        <span>{relationships.length} Relationships</span>
                    </div>
                </div>
            </div>
        </div>
    </PageHeader>

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
                        <div class="flex items-center justify-between px-2">
                            <h3 class="text-xs font-black uppercase tracking-[0.3em] opacity-30 border-l-2 border-primary/20 pl-2">Knowledge Graph</h3>
                            
                            <!-- View Toggle -->
                            <div class="join bg-base-200 p-0.5 rounded-lg border border-base-300">
                                <button 
                                    class="btn btn-[8px] btn-ghost join-item btn-xs h-6 min-h-0 px-2 {viewMode === 'list' ? 'bg-base-100 shadow-sm opacity-100' : 'opacity-40'}"
                                    onclick={() => viewMode = 'list'}
                                >
                                    <Icon icon="lucide:list" class="w-3 h-3" />
                                </button>
                                <button 
                                    class="btn btn-[8px] btn-ghost join-item btn-xs h-6 min-h-0 px-2 {viewMode === 'graph' ? 'bg-base-100 shadow-sm opacity-100' : 'opacity-40'}"
                                    onclick={() => viewMode = 'graph'}
                                >
                                    <Icon icon="lucide:share-2" class="w-3 h-3" />
                                </button>
                            </div>
                        </div>
                        
                        {#if viewMode === 'list'}
                            <div class="space-y-6 animate-in fade-in duration-300">
                                {#each Object.entries(groupedRelationships) as [predicate, rels]}
                                    <div class="space-y-2">
                                        <h4 class="text-[9px] font-bold uppercase opacity-20 px-2 tracking-widest">{predicate}</h4>
                                        <div class="grid grid-cols-1 gap-2">
                                            {#each rels as rel}
                                                {@const isSource = rel.source.id === params.id}
                                                {@const target = isSource ? rel.target : rel.source}
                                                <div class="p-3 bg-base-200 hover:bg-base-300 transition-all rounded-2xl border border-base-300/50">
                                                    <EntityLabel namespace={target.ns} typ={target.typ} id={target.id} />
                                                </div>
                                            {/each}
                                        </div>
                                    </div>
                                {/each}
                            </div>
                        {:else}
                            <div class="animate-in zoom-in-95 duration-300">
                                <EntityGraph 
                                    {relationships} 
                                    centralEntityId={params.id} 
                                    centralTitle={displayTitle}
                                    centralImage={displayImage}
                                    centralIcon={displayIcon}
                                />
                            </div>
                        {/if}
                    </div>
                {/if}

                <!-- Traits -->
                <div class="space-y-4">
                    <h3 class="text-xs font-black uppercase tracking-[0.3em] opacity-30 px-2">Entity Traits</h3>
                    <div class="grid grid-cols-1 gap-3">
                        {#each Object.entries(traits) as [traitId, value]}
                            <!-- Filter out redundant display traits -->
                            {#if !['scry.visual/photo', 'scry.core/name', 'scry.core/subtitle'].includes(traitId)}
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
