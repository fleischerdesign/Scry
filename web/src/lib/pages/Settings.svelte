<script lang="ts">
    import { router } from '../router.svelte';

    let { plugins = [], dashboards = [] } = $props();
    const enrichers = $derived(plugins.filter(p => p.provided_traits && p.provided_traits.length > 0));
</script>

<div class="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500 w-full max-w-4xl">
    <!-- Home Assistant Style Vertical List -->
    <div class="flex flex-col bg-base-200 rounded-3xl overflow-hidden border border-base-300 divide-y divide-base-300/50">
        <button 
            onclick={() => router.navigate('/settings/general')}
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
            onclick={() => router.navigate('/settings/plugins')}
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
            onclick={() => router.navigate('/settings/enrichers')}
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
            onclick={() => router.navigate('/settings/dashboards')}
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
</div>
