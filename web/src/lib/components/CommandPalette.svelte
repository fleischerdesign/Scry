<script lang="ts">
 import { onMount } from 'svelte';
 import { api } from '../api';
 import { router } from '../router.svelte';
 import Icon from '@iconify/svelte';

 let { isOpen = $bindable(false), onAction } = $props();
 let query = $state("");
 let results = $state<any[]>([]);
 let selectedIndex = $state(0);
 let inputElement = $state<HTMLInputElement>();

 const actions = [
  { id: 'nav-overview', label: 'Go to Overview', category: 'Navigation', icon: 'lucide:layout-dashboard', execute: () => router.navigate('/overview') },
  { id: 'nav-timeline', label: 'Go to Timeline', category: 'Navigation', icon: 'lucide:clock', execute: () => router.navigate('/timeline') },
  { id: 'nav-explorer', label: 'Go to Explorer', category: 'Navigation', icon: 'lucide:search', execute: () => router.navigate('/entity') },
  { id: 'nav-laboratory', label: 'Go to Laboratory', category: 'Navigation', icon: 'lucide:flask-conical', execute: () => router.navigate('/lab') },
  { id: 'nav-analytics', label: 'Go to Insights', category: 'Navigation', icon: 'lucide:zap', execute: () => router.navigate('/analytics') },
  { id: 'nav-settings', label: 'Go to Settings', category: 'Navigation', icon: 'lucide:settings', execute: () => router.navigate('/settings') },
 ];

 $effect(() => {
  if (isOpen && inputElement) {
   setTimeout(() => inputElement?.focus(), 50);
  }
 });

 function highlight(text: string, q: string) {
  if (!q || !text) return text;
  const regex = new RegExp(`(${q})`, 'gi');
  return text.replace(regex, '<span class="search-highlight">$1</span>');
 }

 let filteredItems = $derived.by(() => {
  const q = query.toLowerCase();
  
  const staticActions = actions
   .filter(a => a.label.toLowerCase().includes(q) || a.category.toLowerCase().includes(q))
   .map(a => ({ ...a, snippet: "" })); // Static actions don't have snippets
  
  const searchResults = results.map(r => {
   let icon = 'lucide:box';
   if (r.type === 'event') icon = 'lucide:activity';
   
   return {
    id: r.id,
    label: r.display_title || r.label,
    snippet: r.snippet,
    sublabel: r.display_subtitle || r.subtext,
    image: r.display_image,
    icon,
    category: r.type.toUpperCase(),
    execute: () => router.navigate(r.link)
   };
  });

  return [...staticActions, ...searchResults];
 });

 async function search() {
  if (query.trim().length < 2) {
   results = [];
   return;
  }
  try {
   const data = await api.search(query);
   results = data;
  } catch (e) {
   console.error("Search failed", e);
  }
 }

 function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'k' && (e.metaKey || e.ctrlKey)) {
   e.preventDefault();
   isOpen = !isOpen;
  }
  if (!isOpen) return;

  if (e.key === 'ArrowDown') {
   e.preventDefault();
   selectedIndex = (selectedIndex + 1) % filteredItems.length;
  } else if (e.key === 'ArrowUp') {
   e.preventDefault();
   selectedIndex = (selectedIndex - 1 + filteredItems.length) % filteredItems.length;
  } else if (e.key === 'Enter') {
   e.preventDefault();
   const item = filteredItems[selectedIndex];
   if (item) {
    item.execute();
    isOpen = false;
    query = "";
   }
  } else if (e.key === 'Escape') {
   isOpen = false;
  }
 }

 $effect(() => { 
  if (query) search(); 
  else results = [];
 });

 onMount(() => {
  window.addEventListener('keydown', handleKeydown);
  return () => window.removeEventListener('keydown', handleKeydown);
 });
</script>

<style lang="postcss">
 @reference "../../app.css";

 :global(.search-highlight) {
  @apply text-primary font-bold;
  background: transparent;
 }
 
 :global(.active .search-highlight) {
  @apply text-primary-content;
 }

 .result-item {
  @apply flex justify-between items-center py-3 px-5 rounded-xl transition-all duration-200 border border-transparent;
 }

 .result-item.active {
  @apply bg-primary text-primary-content shadow-lg shadow-primary/20 border-primary;
 }
</style>

<dialog class="modal {isOpen ? 'modal-open' : ''}">
 <div class="modal-box p-0 max-w-2xl bg-base-100 shadow-2xl overflow-hidden flex flex-col border border-base-300 rounded-3xl animate-in zoom-in-95 duration-200">
 <!-- Header -->
 <div class="flex items-center px-6 py-5 border-b border-base-200 gap-4 bg-base-200/30">
  <div class="text-primary opacity-60">
   <Icon icon="lucide:search" class="w-5 h-5" />
  </div>
  <input 
  bind:this={inputElement}
  bind:value={query}
  type="text" 
  placeholder="Search entities, events, or actions..." 
  class="input w-full bg-transparent border-none focus:outline-none focus:ring-0 text-lg font-medium placeholder:opacity-60"
  />
  <div class="flex items-center gap-2">
   <kbd class="kbd kbd-sm opacity-70 bg-base-100">ESC</kbd>
  </div>
 </div>

 <!-- Content -->
 <div class="max-h-[60vh] overflow-y-auto p-2">
  {#if filteredItems.length > 0}
  <div class="space-y-1">
   {#each filteredItems as item, i}
    <button 
     class="result-item w-full text-left {selectedIndex === i ? 'active' : ''}"
     onclick={() => { item.execute(); isOpen = false; query = ""; }}
     onmouseenter={() => selectedIndex = i}
    >
     <div class="flex items-center gap-4 overflow-hidden flex-1">
      <!-- Visual: Image or Icon -->
      <div class="shrink-0">
       {#if (item as any).image}
        <div class="w-10 h-10 rounded-xl overflow-hidden shadow-sm bg-base-300">
         <img src={(item as any).image} alt="" class="w-full h-full object-cover" />
        </div>
       {:else}
        <div class="w-10 h-10 rounded-xl flex items-center justify-center {selectedIndex === i ? 'bg-primary-content/20' : 'bg-base-200'} transition-colors">
         <Icon icon={(item as any).icon || 'lucide:circle'} class="w-5 h-5 {selectedIndex === i ? 'text-primary-content' : 'opacity-70'}" />
        </div>
       {/if}
      </div>

      <!-- Labels -->
      <div class="flex flex-col min-w-0 flex-1">
       <div class="flex items-center gap-2">
        <span class="text-xs font-bold tracking-wide {selectedIndex === i ? 'text-primary-content/60' : 'text-base-content/40'}">
         {item.category}
        </span>
        {#if (item as any).sublabel}
         <span class="text-xs truncate {selectedIndex === i ? 'text-primary-content/40' : 'opacity-60'} font-mono">
          • {(item as any).sublabel}
         </span>
        {/if}
       </div>
       <h4 class="font-black text-sm tracking-tight truncate leading-tight mt-0.5">
        {@html highlight(item.label, query)}
       </h4>
       {#if item.snippet && !item.label.toLowerCase().includes(query.toLowerCase())}
        <p class="text-[11px] {selectedIndex === i ? 'text-primary-content/70' : 'opacity-70'} line-clamp-1 mt-0.5 leading-tight">
         {@html item.snippet}
        </p>
       {/if}
      </div>
     </div>

     <!-- Shortcut Hint -->
     {#if selectedIndex === i}
      <div class="flex items-center gap-2 text-primary-content/40 shrink-0 ml-4 animate-in slide-in-from-right-2">
       <span class="text-xs font-bold tracking-wide">Select</span>
       <Icon icon="lucide:corner-down-left" class="w-3 h-3" />
      </div>
     {/if}
    </button>
   {/each}
  </div>
  {:else}
  <div class="py-24 text-center space-y-4 opacity-60">
   <Icon icon="lucide:search-slash" class="w-12 h-12 mx-auto" />
   <p class="text-sm font-bold tracking-wide">No matching entries found</p>
  </div>
  {/if}
 </div>

 <!-- Footer -->
 <div class="p-4 bg-base-200/50 justify-between items-center flex border-t border-base-200 px-6">
  <div class="flex gap-6">
  <div class="flex items-center gap-2">
   <div class="flex gap-1">
    <kbd class="kbd kbd-xs bg-base-100 opacity-60">↑</kbd>
    <kbd class="kbd kbd-xs bg-base-100 opacity-60">↓</kbd>
   </div>
   <span class="text-xs font-bold opacity-70 tracking-wide">Navigate</span>
  </div>
  <div class="flex items-center gap-2">
   <kbd class="kbd kbd-xs bg-base-100 opacity-60">↵</kbd>
   <span class="text-xs font-bold opacity-70 tracking-wide">Select</span>
  </div>
  </div>
  <div class="flex items-center gap-2 opacity-60">
   <Icon icon="lucide:fingerprint" class="w-3 h-3" />
   <span class="text-xs font-bold tracking-wide">Scry Index v1.1</span>
  </div>
 </div>
 </div>
 <form method="dialog" class="modal-backdrop bg-base-content/20 backdrop-blur-sm transition-all">
 <button onclick={() => isOpen = false}>close</button>
 </form>
</dialog>
