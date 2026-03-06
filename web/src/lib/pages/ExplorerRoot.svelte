<script lang="ts">
	import { onMount } from "svelte";
	import { api } from "../api";
	import { router } from "../router.svelte";
	import Icon from "@iconify/svelte";
	import type { ApiNamespace } from "../types/ApiNamespace";

	let namespaces = $state<ApiNamespace[]>([]);
	let loading = $state(true);

	$effect(() => {
		router.title = "DISCOVERY";
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
					onclick={() => router.navigate(`/entity/${ns.name}`)}
					class="group relative p-8 bg-base-200 border border-base-300 rounded-[2.5rem] hover:bg-base-300 transition-all text-left overflow-hidden"
				>
					<!-- Decoration -->
					<div class="absolute -right-4 -top-4 w-24 h-24 bg-primary/5 rounded-full blur-2xl group-hover:bg-primary/10 transition-colors"></div>
					
					<div class="flex items-center justify-between mb-6">
						<div class="w-14 h-14 rounded-3xl bg-base-100 flex items-center justify-center shadow-inner group-hover:scale-110 transition-transform">
							<Icon 
								icon={ns.display_icon || 'lucide:database'} 
								class="w-6 h-6 text-primary" 
							/>
						</div>
						<div class="badge badge-primary badge-outline badge-xs opacity-30 font-mono tracking-tighter uppercase italic">Namespace</div>
					</div>
					
					<h3 class="font-black text-xl uppercase tracking-tighter mb-2 group-hover:text-primary transition-colors italic">{ns.name}</h3>
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
