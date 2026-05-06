<script lang="ts">
	import { onMount } from "svelte";
	import { api } from "../api";
	import { router } from "../router.svelte";
	import type { ApiEntityType } from "../types/ApiEntityType";
	import Icon from "@iconify/svelte";
	import PageHeader from "../components/PageHeader.svelte";

	let { ns } = $derived(router.getParams("/entity/:ns"));
	let types = $state<ApiEntityType[]>([]);
	let loading = $state(true);

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
</script>

<div class="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
	<PageHeader 
		title={ns || 'Namespace'} 
		subtitle="Browse and explore available entity types within this semantic domain."
	/>

	{#if loading}
		<div class="flex justify-center py-32">
			<span class="loading loading-spinner loading-lg text-primary opacity-60"></span>
		</div>
	{:else}
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
			{#each types as type}
				<button 
					onclick={() => router.navigate(`/entity/${ns}/${type.name}`)}
					class="group p-6 bg-base-100 border border-base-300 rounded-3xl hover:border-primary transition-all text-left"
				>
					<div class="flex items-center justify-between mb-4">
						<div class="w-12 h-12 rounded-2xl bg-primary/5 text-primary flex items-center justify-center group-hover:scale-110 transition-transform shadow-sm">
							<Icon icon={type.display_icon || 'lucide:box'} class="w-6 h-6" />
						</div>
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 opacity-0 group-hover:opacity-100 text-primary transition-opacity" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" /></svg>
					</div>
					<h3 class="font-black text-lg tracking-tight mb-1">{type.name}</h3>
					<p class="text-xs opacity-70 leading-relaxed">
						Domain-specific entity classification and behavior definitions.
					</p>
				</button>
			{/each}
		</div>
	{/if}
</div>
