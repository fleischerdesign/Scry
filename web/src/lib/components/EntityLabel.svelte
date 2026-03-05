<script lang="ts">
    import { identityService } from '../services/identity.svelte';
    import { router } from '../router.svelte';

    interface Props {
        namespace: string;
        typ: string;
        id: string;
        inline?: boolean;
    }

    let { namespace, typ, id, inline = false } = $props<Props>();

    // Resolve the display name from our identity service (with auto-loading)
    let displayTitle = $derived(identityService.resolve({ namespace, typ, id }));
    let entity = $derived(identityService.get({ namespace, typ, id }));
    let isUUID = $derived(id.length > 30 && id.includes('-'));
    let isLoaded = $derived(entity !== undefined);

    function navigate() {
        router.navigate(`/entity/${namespace}/${typ}/${id}`);
    }
</script>

{#if inline}
    <button 
        onclick={navigate}
        class="inline-flex items-center gap-1.5 px-1.5 py-0.5 rounded bg-base-200 hover:bg-primary hover:text-primary-content transition-all text-[10px] font-bold uppercase tracking-tight"
        title="{namespace}:{typ}:{id}"
    >
        {#if entity?.display_image}
            <img src={entity.display_image} alt="" class="w-3 h-3 rounded-sm object-cover" />
        {/if}
        <span>{displayTitle}</span>
    </button>
{:else}
    <div class="flex items-center gap-2 group cursor-pointer" onclick={navigate}>
        {#if entity?.display_image}
            <div class="w-6 h-6 rounded-lg shadow-inner overflow-hidden flex-shrink-0">
                <img src={entity.display_image} alt="" class="w-full h-full object-cover" />
            </div>
        {:else}
            <div class="w-6 h-6 rounded-lg bg-base-200 flex items-center justify-center font-bold text-[8px] group-hover:bg-primary/10 group-hover:text-primary transition-colors flex-shrink-0">
                {displayTitle.charAt(0)}
            </div>
        {/if}
            <div class="flex flex-col min-w-0">
                <span class="text-[10px] font-black uppercase tracking-tighter leading-none group-hover:text-primary transition-colors truncate">
                    {displayTitle}
                </span>
                {#if entity?.display_subtitle}
                    <span class="text-[8px] opacity-60 italic leading-tight truncate">
                        {entity.display_subtitle}
                    </span>
                {:else}
                    <span class="text-[8px] opacity-30 font-mono italic leading-none truncate">
                        {typ}: {id.substring(0, 8)}...
                    </span>
                {/if}
            </div>
    </div>
{/if}
