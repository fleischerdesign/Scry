<script lang="ts">
 import { router } from "../../router.svelte";
 import { api } from "../../api";
 import { ui } from "../../ui.svelte";
 import PageHeader from "../../components/PageHeader.svelte";
 import Icon from "@iconify/svelte";
 import ConfigField from "../../components/ConfigField.svelte";
 import Card from "../../components/Card.svelte";
 import PageLoading from "../../components/PageLoading.svelte";
 import type { PluginStatus } from "../../types/PluginStatus";

 const { id } = $derived(router.getParams("/settings/plugins/:id"));
 
 let plugin = $state<PluginStatus | null>(null);
 let loading = $state(true);
 let configData = $state<Record<string, any>>({});
 let saving = $state(false);
 let polling = $state(false);

 async function loadData() {
  if (!id) return;
  loading = true;
  try {
   const plugins = await api.getPlugins();
   plugin = plugins.find(p => p.id === id) || null;
   if (plugin) {
    configData = plugin.config || {};
   }
  } catch (e) {
   console.error("Failed to load plugin details", e);
  } finally {
   loading = false;
  }
 }

 async function saveConfig() {
  if (!id) return;
  saving = true;
  try {
   await api.updatePluginConfig(id, configData);
   ui.notify("Configuration Saved", `${plugin?.name} settings updated successfully.`, "success");
  } catch (e) {
   ui.notify("Save Failed", "Check plugin requirements.", "error");
  } finally {
   saving = false;
  }
 }

 async function triggerPoll() {
  if (!id) return;
  polling = true;
  try {
   const res = await api.pollPlugin(id);
   ui.notify("Poll Complete", `Fetched ${res.events_saved} new events.`, "success");
  } catch (e) {
   ui.notify("Poll Failed", "Plugin execution error.", "error");
  } finally {
   polling = false;
  }
 }

 async function startOAuth() {
  if (!id) return;
  try {
   const res = await api.getPluginAuthUrl(id);
   if (res.auth_url) {
    window.location.href = res.auth_url;
   } else {
    ui.notify("OAuth Error", res.error || "No auth URL provided", "error");
   }
  } catch (e) {
   ui.notify("OAuth Failed", "Could not initiate authentication.", "error");
  }
 }

 $effect(() => {
  if (id) loadData();
 });
</script>

<div class="space-y-8 animate-in fade-in duration-500 w-full max-w-4xl pb-20">
 <PageHeader 
  title={plugin?.name || "Plugin Settings"} 
  subtitle={plugin?.description || "Configure and manage this system extension."}
  onBack={() => router.navigate("/settings/plugins")}
 >
  {#snippet actions()}
   <div class="flex gap-2">
    <button 
     class="btn btn-sm btn-ghost gap-2 border border-base-300 rounded-xl font-bold opacity-60 hover:opacity-100 transition-all"
     onclick={triggerPoll}
     disabled={polling}
    >
     {#if polling}<span class="loading loading-spinner loading-xs"></span>{/if}
     <Icon icon="lucide:refresh-cw" class="w-4 h-4" />
     Force Poll
    </button>
    <button 
     class="btn btn-sm btn-primary gap-2"
     onclick={saveConfig}
     disabled={saving}
    >
     {#if saving}<span class="loading loading-spinner loading-xs"></span>{/if}
     Apply Changes
    </button>
   </div>
  {/snippet}
 </PageHeader>

 {#if loading}
  <PageLoading />
 {:else if plugin}
  <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
   <!-- Configuration Column -->
   <div class="md:col-span-2 space-y-6">
      <Card>
       {#snippet header()}
        <div class="flex items-center justify-between w-full">
         <span class="text-xs font-bold tracking-wide opacity-60">Plugin Configuration</span>
         <div class="badge badge-outline badge-xs opacity-70 font-mono tracking-tighter">ID: {plugin.id}</div>
        </div>
       {/snippet}
      <div class="space-y-6">
       {#if plugin.config_schema}
        {@const schema = JSON.parse(plugin.config_schema)}
        {#if schema.properties}
         {#each Object.entries(schema.properties) as [key, prop]: any}
          <ConfigField 
           {key} 
           schema={prop} 
           bind:value={configData[key]} 
          />
         {/each}
        {/if}
       {:else}
        <div class="py-10 text-center opacity-60 italic text-sm">
         This plugin does not require manual configuration.
        </div>
       {/if}
      </div>
     </Card>

     {#if plugin.capabilities.includes('oauth')}
      <Card>
       {#snippet header()}
        <Icon icon="lucide:key-round" class="w-4 h-4 opacity-70" />
        <span class="text-xs font-bold tracking-wide opacity-60">Identity & Authentication</span>
       {/snippet}
       <div class="flex flex-col md:flex-row items-center justify-between gap-6">
        <div class="space-y-1">
         <h4 class="font-black text-sm">OAuth Connection</h4>
         <p class="text-xs opacity-70">Authorize Scry to access your {plugin.name} account data.</p>
        </div>
        <button 
         class="btn btn-secondary rounded-xl font-bold gap-2 px-8"
         onclick={startOAuth}
        >
         <Icon icon="lucide:external-link" class="w-4 h-4" />
         Connect {plugin.name}
        </button>
       </div>
      </Card>
     {/if}
   </div>

   <!-- Meta / Status Column -->
   <div class="space-y-6">
     <Card>
      {#snippet header()}
       <Icon icon="lucide:info" class="w-4 h-4 opacity-70" />
       <span class="text-xs font-bold tracking-wide opacity-60">Extension Info</span>
      {/snippet}
      <div class="space-y-4">
       <div class="flex justify-between items-center py-2 border-b border-base-300/50">
        <span class="text-xs font-bold opacity-70 ">Version</span>
        <span class="badge badge-ghost font-mono text-xs">{plugin.version}</span>
       </div>
       <div class="flex justify-between items-center py-2 border-b border-base-300/50">
        <span class="text-xs font-bold opacity-70 ">Status</span>
        <span class="flex items-center gap-1.5 text-xs font-bold text-success">
         <div class="w-1.5 h-1.5 rounded-full bg-success animate-pulse"></div>
         Active
        </span>
       </div>
       <div class="flex justify-between items-center py-2">
        <span class="text-xs font-bold opacity-70 ">Polling</span>
        <span class="text-xs font-mono opacity-60">Every 60s</span>
       </div>
      </div>
     </Card>

     <Card>
      {#snippet header()}
       <Icon icon="lucide:layers" class="w-4 h-4 opacity-70" />
       <span class="text-xs font-bold tracking-wide opacity-60">Capabilities</span>
      {/snippet}
      <div class="flex flex-wrap gap-2">
       {#each plugin.capabilities as cap}
        <div class="badge badge-ghost border-base-300 font-mono text-xs uppercase">{cap}</div>
       {/each}
      </div>
     </Card>
   </div>
  </div>
 {/if}
</div>
