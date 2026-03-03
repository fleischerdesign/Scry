<script lang="ts">
	import { onMount } from "svelte";
	import { api } from "../api";
	import { router } from "../router.svelte";

	let namespaces = $state<string[]>([]);
	let loading = $state(true);

	$effect(() => {
		router.title = "KNOWLEDGE GRAPH";
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
	<div class="flex flex-col gap-2">
		<h2 class="text-4xl font-black font-mono tracking-tighter italic uppercase text-base-content">
			Discovery <span class="text-primary opacity-20">/ Root</span>
		</h2>
		<p class="text-[10px] opacity-40 uppercase tracking-[0.2em] font-bold">
			Unified semantic exploration of your digitized life
		</p>
	</div>

	{#if loading}
		<div class="flex justify-center py-32">
			<span class="loading loading-infinity loading-lg text-primary opacity-20"></span>
		</div>
	{:else}
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
			{#each namespaces as ns}
				<button 
					onclick={() => router.navigate(`/entity/${ns}`)}
					class="group relative p-8 bg-base-200 border border-base-300 rounded-[2.5rem] hover:bg-base-300 transition-all text-left overflow-hidden"
				>
					<!-- Decoration -->
					<div class="absolute -right-4 -top-4 w-24 h-24 bg-primary/5 rounded-full blur-2xl group-hover:bg-primary/10 transition-colors"></div>
					
					<div class="flex items-center justify-between mb-6">
						<div class="w-14 h-14 rounded-3xl bg-base-100 flex items-center justify-center shadow-inner group-hover:scale-110 transition-transform">
							<svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 text-primary" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" /></svg>
						</div>
						<div class="badge badge-primary badge-outline badge-xs opacity-30 font-mono tracking-tighter uppercase italic">Namespace</div>
					</div>
					
					<h3 class="font-black text-xl uppercase tracking-tighter mb-2 group-hover:text-primary transition-colors italic">{ns}</h3>
					<p class="text-[10px] opacity-40 uppercase font-mono leading-relaxed">
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
