<script lang="ts">
    import { api } from "../api";
    import { onMount } from "svelte";
    import { router } from "../router.svelte";

    let { item, isFirst = false, isLast = false } = $props();
    let traits = $state<Record<string, any>>({});

    const time = $derived(new Date(item.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }));

    onMount(async () => {
        const entities = item.entities || [];
        if (entities.length > 0) {
            for (const ent of entities) {
                try {
                    const fetchedTraits = await api.getEntityTraits(ent.namespace, ent.typ, ent.id);
                    traits = { ...traits, ...fetchedTraits };
                } catch (e) {
                    console.warn("Failed to fetch traits for entity", ent, e);
                }
            }
        }
    });

    const photoUrl = $derived(traits["scry.visual/photo"]);
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
            {#if photoUrl}
                <div class="avatar">
                    <div class="w-10 h-10 rounded-xl shadow-lg ring ring-primary/10">
                        <img src={photoUrl} alt="Entity visualization" />
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
                    <!-- Metadata / Enrichment Info -->
                    {#if item.metadata}
                        {#each Object.entries(item.metadata) as [metaKey, metaVal]}
                            {#if !metaKey.startsWith('display.')}
                                <div class="badge badge-ghost badge-sm font-mono gap-1">
                                    <span class="opacity-50 text-[10px]">{metaKey}:</span>
                                    <span>{metaVal}</span>
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
