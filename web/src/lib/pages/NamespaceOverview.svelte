<script lang="ts">
	import { onMount } from "svelte";
	import { api } from "../api";
	import { router } from "../router.svelte";
	import { createPluginsQuery } from "../queries/plugins";
	import Icon from "@iconify/svelte";

	let { ns } = $derived(router.getParams("/entity/:ns"));
	let types = $state<string[]>([]);
	let loading = $state(true);

	const pluginsQuery = createPluginsQuery();

	$effect(() => {
		router.title = ns.toUpperCase();
	});

	async function loadTypes() {
		if (!ns) return;
		loading = true;
		try {
			types = await api.getNamespaceTypes(ns);
		} catch (e) {
			console.error(e);
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		if (ns) loadTypes();
	});

	function getIconForType(type: string) {
		const items = pluginsQuery.data ?? [];
		// 1. Suche in allen Plugin-Exports nach diesem semantischen Typ
		for (const p of items) {
			if (p.exports) {
				const exp = p.exports.find(e => e.semantic_type === `${ns}/${type}` || e.semantic_type === type);
				if (exp?.icon) return exp.icon;
			}
		}

		// 2. Fallbacks
		const t = type.toLowerCase();
		if (t.includes('artist') || t.includes('user')) return 'lucide:user';
		if (t.includes('track') || t.includes('song') || t.includes('music')) return 'lucide:music';
		if (t.includes('album')) return 'lucide:disc';
		if (t.includes('location') || t.includes('city') || t.includes('place')) return 'lucide:map-pin';
		if (t.includes('weather') || t.includes('temp')) return 'lucide:thermometer';
		
		return 'lucide:file-question';
	}
</script>

<div class="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
	<div class="flex items-center justify-between">
		<div>
			<h2 class="text-2xl font-black font-mono tracking-tighter italic uppercase">
				<span class="text-primary opacity-40">NAMESPACE /</span> {ns}
			</h2>
			<p class="text-[10px] opacity-40 uppercase tracking-widest font-bold">
				Available entity types within this semantic namespace
			</p>
		</div>
	</div>

	{#if loading}
		<div class="flex justify-center py-32">
			<span class="loading loading-infinity loading-lg text-primary opacity-20"></span>
		</div>
	{:else}
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
			{#each types as type}
				<button 
					onclick={() => router.navigate(`/entity/${ns}/${type}`)}
					class="group p-6 bg-base-100 border border-base-300 rounded-3xl hover:border-primary transition-all text-left"
				>
					<div class="flex items-center justify-between mb-4">
						<div class="w-12 h-12 rounded-2xl bg-primary/5 text-primary flex items-center justify-center group-hover:scale-110 transition-transform shadow-sm">
							<Icon icon={getIconForType(type)} class="w-6 h-6" />
						</div>
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 opacity-0 group-hover:opacity-100 text-primary transition-opacity" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" /></svg>
					</div>
					<h3 class="font-bold text-lg uppercase tracking-tight mb-1">{type}</h3>
					<p class="text-[10px] opacity-40 uppercase font-mono tracking-widest">
						Explore all {type} entities_
					</p>
				</button>
			{/each}
		</div>
	{/if}
</div>
