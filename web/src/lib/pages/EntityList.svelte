<script lang="ts">
	import { onMount } from "svelte";
	import { api } from "../api";
	import { router } from "../router.svelte";
	import Card from "../components/Card.svelte";

	let { onRefresh } = $props();
	let { ns, type } = $derived(router.getParams("/entity/:ns/:type"));
	let entities = $state<any[]>([]);
	let loading = $state(true);

	$effect(() => {
		router.title = type.toUpperCase();
	});

	async function loadEntities() {
		if (!ns || !type) return;
		loading = true;
		try {
			entities = await api.getEntities(ns, type);
		} catch (e) {
			console.error(e);
		} finally {
			loading = false;
		}
	}

	$effect(() => {
		if (ns && type) loadEntities();
	});
</script>

<div class="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
	<div class="flex items-center justify-between">
		<div>
			<h2 class="text-2xl font-black font-mono tracking-tighter italic uppercase">
				{type} <span class="text-primary opacity-40">/ {ns}</span>
			</h2>
			<p class="text-[10px] opacity-40 uppercase tracking-widest font-bold">
				Discovery and exploration of all registered entities
			</p>
		</div>
	</div>

	{#if loading}
		<div class="flex justify-center py-32">
			<span class="loading loading-infinity loading-lg text-primary opacity-20"></span>
		</div>
	{:else}
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
			{#each entities as entity}
				<button 
					onclick={() => router.navigate(entity.link)}
					class="card bg-base-100 border border-base-300 hover:border-primary transition-all group text-left"
				>
					<div class="card-body p-4">
						<div class="flex items-center gap-3">
							<div class="w-10 h-10 rounded-xl bg-base-200 flex items-center justify-center font-bold text-xs group-hover:bg-primary/10 group-hover:text-primary transition-colors">
								{entity.title.charAt(0)}
							</div>
							<div class="flex-1 min-w-0">
								<h3 class="font-bold text-sm truncate uppercase tracking-tight">{entity.title}</h3>
								<p class="text-[10px] opacity-40 font-mono truncate italic">{entity.id}</p>
							</div>
						</div>
					</div>
				</button>
			{:else}
				<div class="col-span-full py-20 text-center opacity-20 italic font-mono text-xs uppercase tracking-widest">
					No entities found for this type.
				</div>
			{/each}
		</div>
	{/if}
</div>
