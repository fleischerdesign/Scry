<script lang="ts">
    import Widget from "../components/Widget.svelte";
    import { api } from '../api';
    import { ui } from '../ui.svelte';
    import { router } from '../router.svelte';
    
    let { dashboards = [], onRefresh } = $props();

    let isEditing = $state(false);
    let isCreating = $state(false);
    let newDashName = $state("");
    let deletingId = $state<string | null>(null);

    // Wir finden das Dashboard basierend auf dem Slug in der URL
    const activeDashboard = $derived.by(() => {
        const slug = router.path.split('/').pop();
        return dashboards.find(d => d.slug === slug) || dashboards[0] || null;
    });

    async function handleCreate() {
        if (!newDashName) return;
        try {
            await api.createDashboard(newDashName);
            ui.notify("Dashboard Created", newDashName, "success");
            newDashName = "";
            isCreating = false;
            onRefresh();
        } catch (e) { console.error(e); }
    }

    async function removeWidget(widgetId: string) {
        if (!activeDashboard) return;
        deletingId = widgetId;
        try {
            await api.deleteWidget(activeDashboard.id, widgetId);
            ui.notify("Widget Removed", "", "info");
            onRefresh();
        } catch (e) { console.error(e); } finally { deletingId = null; }
    }
</script>

<div class="space-y-10 animate-in fade-in duration-500 w-full pb-20">
    <div class="flex items-center justify-between border-b border-base-300 pb-6">
        <div>
            <h2 class="text-3xl font-black font-mono tracking-tighter italic text-secondary">
                {activeDashboard?.name || 'MY_DASHBOARD'}
            </h2>
            <p class="text-[10px] uppercase tracking-[0.4em] opacity-30 mt-2 font-black">Custom Interface Control</p>
        </div>

        <div class="flex gap-2">
            <button 
                class="btn btn-sm font-mono text-[10px] uppercase tracking-widest {isEditing ? 'btn-primary' : 'btn-ghost opacity-50'}"
                onclick={() => isEditing = !isEditing}
            >
                {isEditing ? 'SAVE_CHANGES' : 'EDIT_LAYOUT'}
            </button>
            <button class="btn btn-sm btn-ghost opacity-50 font-mono text-[10px]" onclick={() => isCreating = true}>NEW_BOARD+</button>
        </div>
    </div>

    {#if isCreating}
        <div class="alert bg-base-100 border border-secondary/30 shadow-xl animate-in zoom-in-95 duration-200">
            <div class="flex-1 flex items-center gap-4">
                <input 
                    type="text" bind:value={newDashName} 
                    placeholder="Dashboard Name..." 
                    class="input input-bordered input-sm font-mono flex-1" 
                    autofocus
                />
                <button class="btn btn-secondary btn-sm" onclick={handleCreate}>Create</button>
                <button class="btn btn-ghost btn-sm" onclick={() => isCreating = false}>Cancel</button>
            </div>
        </div>
    {/if}

    {#if activeDashboard && activeDashboard.widgets.length > 0}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            {#each activeDashboard.widgets as widget (widget.id)}
                <div class="relative group h-full">
                    <Widget {widget} />
                    
                    {#if isEditing}
                        <button 
                            class="absolute -top-2 -right-2 btn btn-circle btn-error btn-xs shadow-lg animate-in zoom-in-50 z-10"
                            onclick={() => removeWidget(widget.id)}
                            disabled={deletingId === widget.id}
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M6 18L18 6M6 6l12 12" /></svg>
                        </button>
                    {/if}
                </div>
            {/each}
        </div>
    {:else}
        <div class="flex flex-col items-center justify-center py-40 opacity-20 border-2 border-dashed border-base-300 rounded-3xl">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-16 w-16 mb-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" /></svg>
            <p class="font-mono text-sm uppercase tracking-[0.3em] font-black">Dashboard empty</p>
            <p class="text-xs mt-2">Go to Explorer and pin some metrics to this board.</p>
        </div>
    {/if}
</div>
