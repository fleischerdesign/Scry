<script lang="ts">
	import { onMount } from "svelte";
	import { api } from "../api";
	import { router } from "../router.svelte";
	import Icon from "@iconify/svelte";
	import PageHeader from "../components/PageHeader.svelte";
	import type { ApiNamespace } from "../types/ApiNamespace";

	let namespaces = $state<ApiNamespace[]>([]);
	let loading = $state(true);

	$effect(() => {
		router.title = "EXPLORER";
	});

	async function loadNamespaces() {
		loading = true;
		try {
			namespaces = await api.getNamespaces();
		} catch (e) {
			console.error(e);
		} finally {
			loading = false;
		}
	}

	onMount(loadNamespaces);
</script>

<div class="space-y-10 animate-in fade-in slide-in-from-bottom-4 duration-500">
	<PageHeader 
		title="Explorer" 
		subtitle="Unified semantic exploration of your digitized life across multiple namespaces and domains."
	/>

	{#if loading}
		<div class="flex justify-center py-32">
			<span class="loading loading-infinity loading-lg text-primary opacity-20"></span>
		</div>
	{:else}
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
		{#each namespaces as ns}
			<button 
				onclick={() => router.navigate(`/entity/${ns.name}`)}
				class="group p-6 bg-base-100 border border-base-300 rounded-[2rem] hover:border-primary transition-all text-left"
			>
				<div class="flex items-center justify-between mb-4">
					<div class="w-12 h-12 rounded-2xl bg-primary/5 text-primary flex items-center justify-center group-hover:scale-110 transition-transform shadow-sm">
						<Icon icon={ns.display_icon || 'lucide:database'} class="w-6 h-6" />
					</div>
					<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 opacity-0 group-hover:opacity-100 text-primary transition-opacity" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" /></svg>
				</div>
				<h3 class="font-bold text-lg uppercase tracking-tight mb-1">{ns.name}</h3>
				<p class="text-[10px] opacity-40 uppercase font-mono tracking-widest leading-relaxed">
					Semantic domain containing specialized entity definitions and relationships_
				</p>
			</button>
		{:else}
				<div class="col-span-full py-32 text-center opacity-20 italic font-mono text-xs uppercase tracking-[0.3em]">
					The graph is currently empty. Start ingesting data.
				</div>
			{/each}
		</div>
	{/if}
</div>
