<script lang="ts">
    import { onMount } from 'svelte';
    import { api } from '../api';
    import Card from "../components/Card.svelte";

    let { plugins = [] } = $props();
    
    let profile = $state<Record<string, string>>({});
    let pluginConfigs = $state<Record<string, Record<string, string>>>({});
    let loading = $state(true);
    let saving = $state(false);
    let successMessage = $state("");

    // Initialisiere die Config-Objekte sofort, wenn sich die Plugins ändern
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
            
            // Lade für jedes Plugin mit 'config' capability die werte (optionaler schritt für die zukunft)
            // Aktuell nutzen wir leere defaults falls nicht in DB
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
</script>

<div class="space-y-10 animate-in fade-in duration-500 w-full max-w-4xl">
    <div class="flex items-center justify-between border-b border-base-300 pb-6">
        <div>
            <h2 class="text-3xl font-black font-mono tracking-tighter italic text-secondary">SETTINGS_</h2>
            <p class="text-[10px] uppercase tracking-[0.4em] opacity-30 mt-2 font-black">Identity & Node Configuration</p>
        </div>
        {#if successMessage}
            <div class="badge badge-success font-mono text-[10px] py-3 px-4 animate-bounce">
                {successMessage}
            </div>
        {/if}
    </div>

    {#if loading}
        <div class="flex justify-center py-20">
            <span class="loading loading-ring loading-lg opacity-20"></span>
        </div>
    {:else}
        <!-- Global Profile Section -->
        <section class="space-y-6">
            <div class="flex items-center gap-4 px-2">
                <div class="w-8 h-px bg-base-300"></div>
                <h3 class="text-xs font-black uppercase tracking-widest opacity-40">Shared Identity</h3>
            </div>
            
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

        <!-- Plugin Configurations -->
        <section class="space-y-6 pt-10">
            <div class="flex items-center gap-4 px-2">
                <div class="w-8 h-px bg-base-300"></div>
                <h3 class="text-xs font-black uppercase tracking-widest opacity-40">Node Specific Parameters</h3>
            </div>

            <div class="grid grid-cols-1 gap-6">
                {#each plugins as plugin}
                    {#if plugin.capabilities.includes('config')}
                        <Card title={plugin.name} subtitle={plugin.id}>
                            <div class="py-2">
                                <p class="text-[10px] opacity-40 mb-4 italic">This node requests specific parameters for operation.</p>
                                
                                {#if plugin.id === 'scry_weather_plugin' && pluginConfigs[plugin.id]}
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
                                    <div class="alert alert-ghost border border-base-300 text-[10px] font-mono opacity-50">
                                        <span>No custom UI fields defined for this node. Parameters can be added via API.</span>
                                    </div>
                                {/if}
                            </div>
                            {#snippet actions()}
                                <button class="btn btn-ghost btn-xs font-mono text-secondary" onclick={() => savePluginConfig(plugin.id)} disabled={saving}>
                                    SAVE_NODE_CONFIG
                                </button>
                            {/snippet}
                        </Card>
                    {/if}
                {/each}
            </div>
        </section>
    {/if}
</div>
