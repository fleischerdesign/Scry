<script lang="ts">
    import { onMount } from 'svelte';
    import { api } from '../api';
    import { router } from '../router.svelte';

    let { isOpen = $bindable(false), onAction } = $props();
    let query = $state("");
    let results = $state<any[]>([]);
    let selectedIndex = $state(0);
    let inputElement = $state<HTMLInputElement>();

    const actions = [
        { id: 'nav-overview', label: 'Go to Overview', category: 'Navigation', execute: () => router.navigate('/overview') },
        { id: 'nav-timeline', label: 'Go to Timeline', category: 'Navigation', execute: () => router.navigate('/timeline') },
        { id: 'nav-analytics', label: 'Go to Analytics', category: 'Navigation', execute: () => router.navigate('/analytics') },
        { id: 'nav-settings', label: 'Go to Settings', category: 'Navigation', execute: () => router.navigate('/settings') },
    ];

    $effect(() => {
        if (isOpen && inputElement) {
            setTimeout(() => inputElement?.focus(), 50);
        }
    });

    let filteredItems = $derived.by(() => {
        const q = query.toLowerCase();
        
        const staticActions = actions.filter(a => a.label.toLowerCase().includes(q) || a.category.toLowerCase().includes(q));
        
        const searchResults = results.map(r => {
            let displayLabel = r.content;
            
            // Wenn es ein Event ist, versuchen wir den JSON-Payload schön zu rendern
            if (r.type === 'event') {
                try {
                    // Wir extrahieren den JSON-Teil (alles nach der Kategorie)
                    const jsonStr = r.content.substring(r.title.length).trim();
                    const p = JSON.parse(jsonStr);
                    
                    if (r.title === 'music.scrobble') {
                        displayLabel = `${p.artist || 'Unknown'} - ${p.track || 'Unknown'}`;
                    } else if (r.title === 'weather.current') {
                        displayLabel = `Weather: ${p.temperature}°C`;
                    } else {
                        displayLabel = p.message || p.id || displayLabel;
                    }
                } catch (e) {
                    // Fallback: Falls Parsing fehlschlägt, nutzen wir den gekürzten Content
                    if (displayLabel.length > 80) displayLabel = displayLabel.slice(0, 80) + '...';
                }
            } else {
                // Für Entitäten etc. einfach kürzen
                if (displayLabel.length > 80) displayLabel = displayLabel.slice(0, 80) + '...';
            }

            return {
                id: r.id,
                label: displayLabel,
                sublabel: r.title,
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

<dialog class="modal {isOpen ? 'modal-open' : ''}">
  <div class="modal-box p-0 max-w-2xl bg-base-100 shadow-2xl overflow-hidden flex flex-col border border-base-300">
    <!-- Header -->
    <div class="flex items-center px-6 py-4 border-b border-base-200 gap-4">
      <div class="text-primary"><svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" /></svg></div>
      <input 
        bind:this={inputElement}
        bind:value={query}
        type="text" 
        placeholder="Search for events, entities, or navigation..." 
        class="input w-full bg-transparent border-none focus:outline-none focus:ring-0 text-lg font-mono placeholder:opacity-20"
      />
      <kbd class="kbd kbd-sm font-mono opacity-50">ESC</kbd>
    </div>

    <!-- Content -->
    <div class="max-h-[60vh] overflow-y-auto">
      {#if filteredItems.length > 0}
        <ul class="menu w-full p-0">
            {#each filteredItems as item, i}
            <li class="w-full">
                <button 
                class="flex justify-between items-center py-4 px-6 rounded-none border-b border-base-200/50 w-full {selectedIndex === i ? 'active bg-primary/10 text-primary' : ''}"
                onclick={() => { item.execute(); isOpen = false; query = ""; }}
                onmouseenter={() => selectedIndex = i}
                >
                <div class="flex flex-col items-start overflow-hidden flex-1">
                    <div class="flex items-center gap-2">
                        <span class="text-[9px] px-1.5 py-0.5 rounded-md bg-base-300 font-black uppercase tracking-widest text-base-content/50">{item.category}</span>
                        {#if (item as any).sublabel}
                            <span class="text-[10px] opacity-40 font-mono italic">{(item as any).sublabel}</span>
                        {/if}
                    </div>
                    <span class="font-bold tracking-tight mt-1 truncate w-full text-left">{item.label}</span>
                </div>
                {#if selectedIndex === i}
                    <div class="flex items-center gap-2 opacity-40 shrink-0 ml-4">
                        <span class="text-[10px] font-mono">NAVIGATE</span>
                        <span class="text-[10px] px-1.5 py-0.5 rounded bg-base-300">↵</span>
                    </div>
                {/if}
                </button>
            </li>
            {/each}
        </ul>
      {:else}
        <div class="py-20 text-center space-y-4">
            <div class="text-4xl opacity-10">🔍</div>
            <p class="text-xs font-mono opacity-20 uppercase tracking-[0.3em]">No matching entries found</p>
        </div>
      {/if}
    </div>

    <!-- Footer -->
    <div class="p-4 bg-base-200/50 justify-between items-center flex border-t border-base-200">
      <div class="flex gap-4">
        <div class="flex items-center gap-1.5">
            <kbd class="kbd kbd-xs bg-base-300">↑</kbd>
            <kbd class="kbd kbd-xs bg-base-300">↓</kbd>
            <span class="text-[10px] opacity-40 uppercase font-bold">Navigate</span>
        </div>
        <div class="flex items-center gap-1.5">
            <kbd class="kbd kbd-xs bg-base-300">↵</kbd>
            <span class="text-[10px] opacity-40 uppercase font-bold">Select</span>
        </div>
      </div>
      <span class="text-[9px] font-black font-mono opacity-30 uppercase tracking-widest">Scry Universal Index v1.0</span>
    </div>
  </div>
  <form method="dialog" class="modal-backdrop">
    <button onclick={() => isOpen = false}>close</button>
  </form>
</dialog>
