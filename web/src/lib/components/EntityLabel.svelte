<script lang="ts">
 import { identityService } from '../services/identity.svelte';
 import { router } from '../router.svelte';
 import Icon from '@iconify/svelte';

 let { namespace, typ, id, inline = false } = $props<{ namespace: string; typ: string; id: string; inline?: boolean }>();

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
  class="inline-flex items-center gap-1.5 px-1.5 py-0.5 rounded bg-base-200 hover:bg-primary hover:text-primary-content transition-all text-xs font-bold tracking-tight"
  title="{namespace}:{typ}:{id}"
 >
  {#if entity?.display_image}
   <img src={entity.display_image} alt="" class="w-3 h-3 rounded-sm object-cover" />
  {:else if entity?.display_icon}
   <Icon icon={entity.display_icon} class="w-3 h-3 opacity-60" />
  {/if}
  <span>{displayTitle}</span>
 </button>
{:else}
 <div class="flex items-center gap-2 group cursor-pointer" onclick={navigate}>
  {#if entity?.display_image}
   <div class="w-6 h-6 rounded-lg shadow-inner overflow-hidden flex-shrink-0">
    <img src={entity.display_image} alt="" class="w-full h-full object-cover" />
   </div>
  {:else if entity?.display_icon}
   <div class="w-6 h-6 rounded-lg bg-base-200 flex items-center justify-center group-hover:bg-primary/10 group-hover:text-primary transition-colors flex-shrink-0">
    <Icon icon={entity.display_icon} class="w-3.5 h-3.5 opacity-70 group-hover:opacity-100 transition-opacity" />
   </div>
  {:else}
   <div class="w-6 h-6 rounded-lg bg-base-200 flex items-center justify-center font-bold text-xs group-hover:bg-primary/10 group-hover:text-primary transition-colors flex-shrink-0">
    {displayTitle.charAt(0)}
   </div>
  {/if}
   <div class="flex flex-col min-w-0">
    <span class="text-xs font-bold tracking-tighter leading-none group-hover:text-primary transition-colors truncate">
     {displayTitle}
    </span>
    {#if entity?.display_subtitle}
     <span class="text-xs opacity-60 leading-tight truncate">
      {entity.display_subtitle}
     </span>
    {:else}
     <span class="text-xs opacity-60 font-mono leading-none truncate">
      {typ}: {id.substring(0, 8)}...
     </span>
    {/if}
   </div>
 </div>
{/if}
