<script lang="ts">
    import Widget from "../components/Widget.svelte";
    import { api } from '../api';
    import { ui } from '../ui.svelte';
    import { router } from '../router.svelte';
    import { dashboards } from "../state/dashboards.svelte";
    import { plugins } from "../state/plugins.svelte";
    
    let { onRefresh } = $props();

    let isEditing = $state(false);
    let isCreating = $state(false);
    let isAddingWidget = $state(false);
    let newDashName = $state("");
    let deletingId = $state<string | null>(null);

    // Find active dashboard from URL via global state
    const activeDashboard = $derived(dashboards.active);

    // Collect all suggested widgets from global plugin state
    const widgetMarketplace = $derived.by(() => {
        return plugins.items.flatMap(p => (p.suggested_widgets || []).map((w: any) => ({
            ...w,
            pluginName: p.name,
            pluginId: p.id
        })));
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

    async function addSuggestedWidget(w: any) {
        if (!activeDashboard) return;
        try {
            const config = typeof w.config_json === 'string' ? JSON.parse(w.config_json) : w.config_json;
            await api.addWidget(activeDashboard.id, {
                type: w.template,
                title: w.title,
                width_span: w.template === 'Trend' ? 2 : 1,
                config: config
            });
            ui.notify("Widget Added", w.title, "success");
            isAddingWidget = false;
            onRefresh();
        } catch (e) { console.error("Failed to add widget", e); }
    }
</script>

<div class="space-y-10 animate-in fade-in duration-500 w-full pb-20">
    <div class="flex items-center justify-between border-b border-base-300 pb-6">
        <div>
            <div class="flex items-center gap-3">
                <h2 class="text-3xl font-black font-mono tracking-tighter italic text-secondary uppercase">
                    {activeDashboard?.name || 'MY_DASHBOARD'}_
                </h2>
                {#if isEditing}
                    <div class="badge badge-primary badge-xs font-mono animate-pulse">EDIT_MODE</div>
                {/if}
            </div>
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

    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {#if activeDashboard}
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
        {/if}

        <!-- Add Widget Placeholder (Only in Edit Mode) -->
        {#if isEditing}
            <button 
                onclick={() => isAddingWidget = true}
                class="h-48 border-2 border-dashed border-primary/20 hover:border-primary/50 hover:bg-primary/5 transition-all rounded-3xl flex flex-col items-center justify-center gap-3 group"
            >
                <div class="w-10 h-10 rounded-2xl bg-primary/10 flex items-center justify-center text-primary group-hover:scale-110 transition-transform">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M12 4v16m8-8H4" /></svg>
                </div>
                <span class="text-[10px] font-black uppercase tracking-widest opacity-40 group-hover:opacity-100">Add Widget</span>
            </button>
        {/if}
    </div>

    {#if !activeDashboard || (activeDashboard.widgets.length === 0 && !isEditing)}
        <div class="flex flex-col items-center justify-center py-40 opacity-20 border-2 border-dashed border-base-300 rounded-3xl">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-16 w-16 mb-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" /></svg>
            <p class="font-mono text-sm uppercase tracking-[0.3em] font-black">Dashboard empty</p>
            <p class="text-xs mt-2">Enter Edit Mode to add suggested widgets from your nodes.</p>
        </div>
    {/if}
</div>

<!-- Widget Marketplace Modal -->
{#if isAddingWidget}
    <div class="modal modal-open">
        <div class="modal-box max-w-2xl bg-base-100 border border-base-300 rounded-3xl p-0 overflow-hidden">
            <div class="p-6 border-b border-base-200 flex justify-between items-center bg-base-200/50">
                <div>
                    <h3 class="font-black text-xl tracking-tight">Widget Marketplace</h3>
                    <p class="text-[10px] uppercase opacity-40 tracking-widest font-bold">Suggested by active nodes</p>
                </div>
                <button class="btn btn-sm btn-circle btn-ghost" onclick={() => isAddingWidget = false}>✕</button>
            </div>
            
            <div class="p-4 grid grid-cols-1 md:grid-cols-2 gap-4 max-h-[60vh] overflow-y-auto">
                {#each widgetMarketplace as w}
                    <button 
                        onclick={() => addSuggestedWidget(w)}
                        class="flex flex-col items-start p-5 bg-base-200 hover:bg-base-300 transition-all rounded-2xl border border-transparent hover:border-primary/20 group text-left"
                    >
                        <div class="flex justify-between w-full items-start mb-3">
                            <div class="badge badge-outline badge-xs font-mono opacity-40 uppercase">{w.pluginName}</div>
                            <div class="badge badge-primary badge-xs font-mono uppercase">{w.template}</div>
                        </div>
                        <span class="font-bold text-sm truncate w-full">{w.title}</span>
                        <span class="text-[9px] opacity-40 mt-1 uppercase tracking-tighter">Click to install recipe</span>
                    </button>
                {:else}
                    <div class="col-span-2 py-20 text-center opacity-20">
                        <p class="font-mono text-xs uppercase tracking-widest">No suggested widgets found</p>
                    </div>
                {/each}
            </div>

            <div class="p-4 bg-base-200/50 border-t border-base-200 text-center">
                <p class="text-[9px] font-black uppercase opacity-30 tracking-[0.2em]">More widgets coming soon from your local nodes</p>
            </div>
        </div>
        <button class="modal-backdrop" onclick={() => isAddingWidget = false}>close</button>
    </div>
{/if}
