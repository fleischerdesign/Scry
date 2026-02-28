<script lang="ts">
    import { onMount } from 'svelte';
    import { api } from '../api';

    let { isOpen = $bindable(false), onAction } = $props();
    let query = $state("");
    let results = $state<any[]>([]);
    let selectedIndex = $state(0);

    const actions = [
        { id: 'nav-overview', label: 'Go to Overview', category: 'Navigation', execute: () => onAction('nav', 'overview') },
        { id: 'nav-timeline', label: 'Go to Timeline', category: 'Navigation', execute: () => onAction('nav', 'timeline') },
        { id: 'nav-analytics', label: 'Go to Analytics', category: 'Navigation', execute: () => onAction('nav', 'analytics') },
        { id: 'nav-system', label: 'Go to System Settings', category: 'Navigation', execute: () => onAction('nav', 'system') },
        { id: 'nav-settings', label: 'Go to User Settings', category: 'Navigation', execute: () => onAction('nav', 'settings') },
        { id: 'poll-music', label: 'Sync Music Node', category: 'Actions', execute: () => onAction('poll', 'scry_music_plugin') },
        { id: 'poll-weather', label: 'Sync Weather Node', category: 'Actions', execute: () => onAction('poll', 'scry_weather_plugin') },
    ];

    let filteredItems = $derived.by(() => {
        const q = query.toLowerCase();
        if (!q) return actions;
        return [
            ...actions.filter(a => a.label.toLowerCase().includes(q) || a.category.toLowerCase().includes(q)),
            ...results.map(r => ({
                id: r.id,
                label: r.event.track || r.event.temperature || r.category,
                category: 'Search Result',
                execute: () => console.log("Selected result", r)
            }))
        ];
    });

    async function search() {
        if (query.length < 2) {
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

    $effect(() => { if (query) search(); });

    onMount(() => {
        window.addEventListener('keydown', handleKeydown);
        return () => window.removeEventListener('keydown', handleKeydown);
    });
</script>

<dialog class="modal {isOpen ? 'modal-open' : ''}">
  <div class="modal-box p-0 max-w-2xl bg-base-100 shadow-2xl overflow-hidden flex flex-col">
    <!-- Header -->
    <div class="flex items-center px-4 py-2 border-b border-base-200">
      <input 
        bind:value={query}
        type="text" 
        placeholder="Search..." 
        class="input w-full bg-transparent border-none focus:outline-none focus:ring-0 text-lg"
        autofocus
      />
      <kbd class="kbd kbd-sm font-mono opacity-50">ESC</kbd>
    </div>

    <!-- Content -->
    <div class="max-h-[60vh] overflow-y-auto">
      <ul class="menu p-2">
        {#each filteredItems as item, i}
          <li>
            <button 
              class="flex justify-between {selectedIndex === i ? 'active' : ''}"
              onclick={() => { item.execute(); isOpen = false; query = ""; }}
              onmouseenter={() => selectedIndex = i}
            >
              <div class="flex flex-col items-start">
                <span class="text-xs opacity-50 uppercase font-bold">{item.category}</span>
                <span class="font-medium">{item.label}</span>
              </div>
              {#if selectedIndex === i}
                <span class="text-[10px] opacity-50">ENTER ↵</span>
              {/if}
            </button>
          </li>
        {/each}
      </ul>
    </div>

    <!-- Footer -->
    <div class="modal-action mt-0 p-4 bg-base-200 justify-between items-center flex">
      <span class="text-[10px] font-mono opacity-50">SCRY KERNEL COMMAND PALETTE</span>
      <form method="dialog">
        <button class="btn btn-sm btn-ghost" onclick={() => isOpen = false}>Close</button>
      </form>
    </div>
  </div>
  <form method="dialog" class="modal-backdrop">
    <button onclick={() => isOpen = false}>close</button>
  </form>
</dialog>
