<script lang="ts">
    import { onMount } from 'svelte';
    import { api } from '../api';
    import Card from "../components/Card.svelte";

    let { plugins = [], dashboards = [], onRefresh } = $props();
    
    // Internal Navigation State
    let currentView = $state("overview"); // overview, general, plugins, enrichers, dashboards
    
    let profile = $state<Record<string, string>>({});
    let newDashboardName = $state("");
    let pluginConfigs = $state<Record<string, Record<string, string>>>({});
    let loading = $state(true);
    let saving = $state(false);
    let successMessage = $state("");

    // Initialisiere die Config-Objekte sofort
    $effect(() => {
        plugins.forEach(p => {
            if (!pluginConfigs[p.id]) {
                pluginConfigs[p.id] = {};
            }
        });
    });

    async function loadData() {
        loading = true;
        try {
            const data = await api.getProfile();
            profile = data;
        } catch (e) {
            console.error("Failed to load settings", e);
        } finally {
            loading = false;
        }
    }

    async function saveProfile() {
        saving = true;
        successMessage = "";
        try {
            await api.updateProfile(profile);
            successMessage = "Profile updated successfully";
            setTimeout(() => successMessage = "", 3000);
        } catch (e) {
            console.error("Failed to save profile", e);
        } finally {
            saving = false;
        }
    }

    async function savePluginConfig(pluginId: string) {
        saving = true;
        successMessage = "";
        try {
            await api.updatePluginConfig(pluginId, pluginConfigs[pluginId]);
            successMessage = `${pluginId} configuration updated`;
            setTimeout(() => successMessage = "", 3000);
        } catch (e) {
            console.error(`Failed to save config for ${pluginId}`, e);
        } finally {
            saving = false;
        }
    }

    onMount(loadData);

    const enrichers = $derived(plugins.filter(p => p.provided_traits && p.provided_traits.length > 0));
</script>

<div class="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500 w-full max-w-4xl">
    <!-- Header & Breadcrumbs -->
    <div class="flex flex-col gap-4 border-b border-base-300 pb-6">
        <div class="text-xs breadcrumbs font-mono opacity-40 uppercase tracking-widest">
            <ul>
                <li><button onclick={() => currentView = "overview"}>Settings</button></li>
                {#if currentView !== "overview"}
                    <li>{currentView}</li>
                {/if}
            </ul>
        </div>
        
        <div class="flex items-center justify-between">
            <div>
                <h2 class="text-3xl font-black font-mono tracking-tighter italic text-secondary uppercase">{currentView}_</h2>
                <p class="text-[10px] uppercase tracking-[0.4em] opacity-30 mt-1 font-black">System Control & Node Configuration</p>
            </div>
            {#if successMessage}
                <div class="badge badge-success font-mono text-[10px] py-3 px-4 animate-bounce">
                    {successMessage}
                </div>
            {/if}
        </div>
    </div>

    {#if loading}
        <div class="flex justify-center py-20">
            <span class="loading loading-ring loading-lg opacity-20"></span>
        </div>
    {:else}
        {#if currentView === "overview"}
            <!-- Home Assistant Style Vertical List -->
            <div class="flex flex-col bg-base-200 rounded-3xl overflow-hidden border border-base-300 divide-y divide-base-300/50">
                <button 
                    onclick={() => currentView = "general"}
                    class="flex items-center gap-4 p-5 hover:bg-base-300 transition-all group text-left w-full"
                >
                    <div class="w-10 h-10 rounded-2xl bg-primary/10 flex items-center justify-center text-primary group-hover:scale-105 transition-transform">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" /></svg>
                    </div>
                    <div class="flex-1">
                        <span class="font-bold text-sm uppercase tracking-tight block">General</span>
                        <span class="text-[10px] opacity-40 uppercase">Profile, identity and global location.</span>
                    </div>
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 opacity-20 group-hover:opacity-100 transition-opacity" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" /></svg>
                </button>

                <button 
                    onclick={() => currentView = "plugins"}
                    class="flex items-center gap-4 p-5 hover:bg-base-300 transition-all group text-left w-full"
                >
                    <div class="w-10 h-10 rounded-2xl bg-secondary/10 flex items-center justify-center text-secondary group-hover:scale-105 transition-transform">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" /></svg>
                    </div>
                    <div class="flex-1">
                        <span class="font-bold text-sm uppercase tracking-tight block">Plugins</span>
                        <span class="text-[10px] opacity-40 uppercase">{plugins.length} active nodes providing data.</span>
                    </div>
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 opacity-20 group-hover:opacity-100 transition-opacity" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" /></svg>
                </button>

                <button 
                    onclick={() => currentView = "enrichers"}
                    class="flex items-center gap-4 p-5 hover:bg-base-300 transition-all group text-left w-full"
                >
                    <div class="w-10 h-10 rounded-2xl bg-accent/10 flex items-center justify-center text-accent group-hover:scale-105 transition-transform">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" /></svg>
                    </div>
                    <div class="flex-1">
                        <span class="font-bold text-sm uppercase tracking-tight block">Enrichers</span>
                        <span class="text-[10px] opacity-40 uppercase">{enrichers.length} semantic processors active.</span>
                    </div>
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 opacity-20 group-hover:opacity-100 transition-opacity" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" /></svg>
                </button>

                <button 
                    onclick={() => currentView = "dashboards"}
                    class="flex items-center gap-4 p-5 hover:bg-base-300 transition-all group text-left w-full"
                >
                    <div class="w-10 h-10 rounded-2xl bg-warning/10 flex items-center justify-center text-warning group-hover:scale-105 transition-transform">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z" /></svg>
                    </div>
                    <div class="flex-1">
                        <span class="font-bold text-sm uppercase tracking-tight block">Dashboards</span>
                        <span class="text-[10px] opacity-40 uppercase">{dashboards.length} UI layouts configured.</span>
                    </div>
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 opacity-20 group-hover:opacity-100 transition-opacity" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" /></svg>
                </button>
            </div>

        {:else if currentView === "general"}
            <section class="space-y-6 animate-in slide-in-from-right-4 duration-300">
                <Card title="User Profile" subtitle="CORE_IDENTITY">
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6 py-2">
                        <div class="form-control w-full">
                            <label class="label" for="profile-name">
                                <span class="label-text text-[10px] font-bold uppercase opacity-50">Display Name</span>
                            </label>
                            <input 
                                type="text" id="profile-name"
                                bind:value={profile['identity.name']} 
                                placeholder="Your Name" 
                                class="input input-bordered font-mono text-sm" 
                            />
                        </div>
                        <div class="form-control w-full">
                            <label class="label" for="profile-city">
                                <span class="label-text text-[10px] font-bold uppercase opacity-50">Home City (Global)</span>
                            </label>
                            <input 
                                type="text" id="profile-city"
                                bind:value={profile['location.city']} 
                                placeholder="Berlin, London..." 
                                class="input input-bordered font-mono text-sm" 
                            />
                        </div>
                    </div>
                    {#snippet actions()}
                        <button class="btn btn-primary btn-xs font-mono" onclick={saveProfile} disabled={saving}>
                            {saving ? 'SAVING...' : 'SYNC_PROFILE'}
                        </button>
                    {/snippet}
                </Card>
            </section>

        {:else if currentView === "plugins"}
            <section class="space-y-4 animate-in slide-in-from-right-4 duration-300">
                {#each plugins as plugin}
                    <div class="collapse collapse-arrow bg-base-200 rounded-3xl border border-base-300">
                        <input type="checkbox" /> 
                        <div class="collapse-title flex items-center gap-4 py-4">
                            <div class="w-8 h-8 rounded-xl bg-base-300 flex items-center justify-center text-xs font-bold font-mono">
                                {plugin.name.charAt(0)}
                            </div>
                            <div>
                                <h3 class="font-bold text-sm uppercase tracking-tight">{plugin.name}</h3>
                                <p class="text-[10px] opacity-40 font-mono italic">{plugin.id} v{plugin.version}</p>
                            </div>
                        </div>
                        <div class="collapse-content space-y-4 px-6 pb-6">
                            <p class="text-xs opacity-60 leading-relaxed">{plugin.description}</p>
                            
                            <div class="flex flex-wrap gap-2">
                                {#each plugin.capabilities as cap}
                                    <div class="badge badge-outline badge-xs font-mono opacity-50">{cap}</div>
                                {/each}
                            </div>

                            {#if plugin.capabilities.includes('config')}
                                <div class="divider opacity-10"></div>
                                <div class="space-y-4">
                                    <h4 class="text-[10px] font-black uppercase tracking-widest opacity-40">Configuration</h4>
                                    
                                    {#if plugin.id === 'scry-weather-plugin' && pluginConfigs[plugin.id]}
                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                            <div class="form-control">
                                                <label class="label" for="{plugin.id}-lat"><span class="label-text text-[10px] font-bold opacity-50">LATITUDE</span></label>
                                                <input type="text" id="{plugin.id}-lat" bind:value={pluginConfigs[plugin.id]['latitude']} placeholder="52.52" class="input input-bordered input-sm font-mono" />
                                            </div>
                                            <div class="form-control">
                                                <label class="label" for="{plugin.id}-lon"><span class="label-text text-[10px] font-bold opacity-50">LONGITUDE</span></label>
                                                <input type="text" id="{plugin.id}-lon" bind:value={pluginConfigs[plugin.id]['longitude']} placeholder="13.41" class="input input-bordered input-sm font-mono" />
                                            </div>
                                        </div>
                                    {:else}
                                        <p class="text-[10px] italic opacity-30">No UI configuration template defined. Configurable via API.</p>
                                    {/if}
                                    
                                    <button class="btn btn-secondary btn-xs font-mono" onclick={() => savePluginConfig(plugin.id)} disabled={saving}>
                                        UPDATE_NODE
                                    </button>
                                </div>
                            {/if}
                        </div>
                    </div>
                {/each}
            </section>

        {:else if currentView === "enrichers"}
            <section class="space-y-4 animate-in slide-in-from-right-4 duration-300">
                <div class="alert bg-primary/5 border border-primary/10 text-xs leading-relaxed">
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="stroke-primary shrink-0 w-6 h-6"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
                    <span>Enricher nodes automatically augment your data with semantic traits like photos, biographies, or location data.</span>
                </div>

                {#each enrichers as enricher}
                    <div class="bg-base-200 p-6 rounded-3xl border border-base-300 space-y-4">
                        <div class="flex items-center justify-between">
                            <div class="flex items-center gap-3">
                                <div class="w-8 h-8 rounded-xl bg-accent/20 text-accent flex items-center justify-center">
                                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" /></svg>
                                </div>
                                <h3 class="font-bold text-sm uppercase tracking-tight">{enricher.name}</h3>
                            </div>
                            <div class="badge badge-accent badge-outline badge-xs font-mono uppercase opacity-50 italic">Active</div>
                        </div>

                        <div class="space-y-2">
                            <h4 class="text-[9px] font-black uppercase tracking-widest opacity-30">Provided Traits</h4>
                            <div class="flex flex-wrap gap-2">
                                {#each enricher.provided_traits as trait}
                                    <div class="badge badge-ghost badge-sm gap-2 font-mono border border-base-300">
                                        <span class="opacity-40">{trait.entity_namespace}/{trait.entity_type}</span>
                                        <span class="text-primary">→</span>
                                        <span class="font-bold">{trait.trait_id.split('/').pop()}</span>
                                    </div>
                                {/each}
                            </div>
                        </div>
                    </div>
                {/each}
            </section>
        {:else if currentView === "dashboards"}
            <section class="space-y-6 animate-in slide-in-from-right-4 duration-300">
                <Card title="Create New Dashboard" subtitle="LAYOUT_ENGINE">
                    <div class="flex gap-4 py-2">
                        <input 
                            type="text" 
                            bind:value={newDashboardName} 
                            placeholder="Enter Dashboard Name..." 
                            class="input input-bordered font-mono text-sm flex-1" 
                        />
                        <button 
                            class="btn btn-primary btn-sm" 
                            onclick={async () => {
                                if (!newDashboardName) return;
                                await api.createDashboard(newDashboardName);
                                newDashboardName = "";
                                if (onRefresh) onRefresh();
                            }}
                        >
                            CREATE
                        </button>
                    </div>
                </Card>

                <div class="space-y-2">
                    <h4 class="text-[10px] font-black uppercase tracking-widest opacity-40 px-2">Existing Dashboards</h4>
                    <div class="flex flex-col bg-base-200 rounded-3xl overflow-hidden border border-base-300 divide-y divide-base-300/50">
                        {#each dashboards as dash}
                            <div class="flex items-center justify-between p-4 px-6">
                                <div class="flex items-center gap-4">
                                    <div class="w-2 h-2 rounded-full bg-secondary"></div>
                                    <span class="font-bold text-sm tracking-tight">{dash.name}</span>
                                    <span class="text-[10px] font-mono opacity-30 italic">/{dash.slug}</span>
                                </div>
                                <button class="btn btn-ghost btn-xs text-error opacity-40 hover:opacity-100">DELETE</button>
                            </div>
                        {/each}
                    </div>
                </div>
            </section>
        {/if}
    {/if}
</div>
