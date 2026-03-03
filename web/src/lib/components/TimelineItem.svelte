<script lang="ts">
    import { router } from "../router.svelte";
    import { semanticService } from "../services/semantic.svelte";
    import type { Event } from "../types/Event";

    let { item, isFirst = false, isLast = false }: { item: Event, isFirst?: boolean, isLast?: boolean } = $props();

    const time = $derived(new Date(item.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }));

    // Agnostically find all metrics in context to show as badges
    const contextMetrics = $derived(semanticService.getMetricsFromContext(item.context_info));
</script>

<li>
    {#if !isFirst}<hr class="bg-base-300" />{/if}
    <div class="timeline-start font-mono text-[10px] opacity-40">{time}</div>
    <div class="timeline-middle">
        <div class="w-3 h-3 rounded-full bg-primary/40 ring-4 ring-primary/10"></div>
    </div>
    <button 
        onclick={() => router.navigate(`/event/${item.id}`)}
        class="timeline-end timeline-box border-none bg-base-100 shadow-none py-4 px-2 hover:bg-base-300/30 transition-colors rounded-xl w-full text-left flex items-center"
    >
        <div class="flex items-center gap-4 w-full">
            {#if item.display_image}
                <div class="avatar">
                    <div class="w-10 h-10 rounded-xl shadow-lg ring ring-primary/10">
                        <img src={item.display_image} alt="Entity visualization" />
                    </div>
                </div>
            {/if}
            
            <div class="flex flex-col gap-1 flex-1 overflow-hidden">
                <div class="flex justify-between items-start">
                    <div class="flex flex-col flex-1">
                        <span class="text-xs font-black tracking-tight truncate">
                            {item.display_title || item.category}
                        </span>
                        {#if item.display_subtitle}
                            <span class="text-[10px] opacity-40 font-mono italic truncate">
                                {item.display_subtitle}
                            </span>
                        {/if}
                    </div>
                    <span class="badge badge-ghost badge-xs font-mono opacity-30 italic lowercase shrink-0 ml-2">
                        {item.category?.split('.').pop()}
                    </span>
                </div>
                
                <div class="flex flex-wrap gap-1 mt-1">
                    <!-- Context Data (Agnostic Metrics) -->
                    {#each contextMetrics as metric}
                        <div 
                            role="button"
                            tabindex="0"
                            onclick={(e) => { 
                                if (metric.source_id) {
                                    e.stopPropagation();
                                    router.navigate(`/event/${metric.source_id}`);
                                }
                            }}
                            onkeydown={(e) => e.key === 'Enter' && metric.source_id && router.navigate(`/event/${metric.source_id}`)}
                            class="badge badge-accent/10 text-accent border-accent/20 badge-xs gap-1 font-black {metric.source_id ? 'hover:bg-accent/20 cursor-pointer transition-colors' : ''}"
                        >
                            <span class="opacity-40 uppercase text-[8px] tracking-widest">{semanticService.getLabel(metric.key)}:</span>
                            {semanticService.formatValue(metric.value?.value ?? metric.value, { semantic_type: metric.key, unit: metric.value?.unit })}
                        </div>
                    {/each}

                    <!-- Context Hints (Aliases) -->
                    {#if item.context && item.context.length > 0}
                        {#each item.context as hint}
                            <div class="badge badge-primary/10 text-primary border-primary/20 badge-xs gap-1 font-black italic tracking-tighter">
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-2 w-2" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" /></svg>
                                {hint.replace('alias:', '').toUpperCase()}
                            </div>
                        {/each}
                    {/if}

                    <!-- Linked Entities -->
                    {#if item.entities && item.entities.length > 0}
                        {#each item.entities as ent}
                            <!-- Wir überspringen 'self' in der Entity-Liste, da es schon im Context steht -->
                            {#if ent.id !== 'self'}
                                <div 
                                    role="button"
                                    tabindex="0"
                                    onclick={(e) => { e.stopPropagation(); router.navigate(`/entity/${ent.namespace}/${ent.typ}/${ent.id}`); }}
                                    onkeydown={(e) => e.key === 'Enter' && router.navigate(`/entity/${ent.namespace}/${ent.typ}/${ent.id}`)}
                                    class="badge badge-secondary/10 text-secondary border-secondary/20 badge-xs gap-1 font-mono hover:bg-secondary/20 transition-colors cursor-pointer"
                                >
                                    <span class="opacity-40">{ent.typ}:</span>
                                    <span class="font-bold">{ent.id}</span>
                                </div>
                            {/if}
                        {/each}
                    {/if}
                </div>
            </div>
        </div>
    </button>
    {#if !isLast}<hr class="bg-base-300" />{/if}
</li>
