<script lang="ts">
	import { onMount } from "svelte";
	import { api } from "../api";
	import { router } from "../router.svelte";
	import type { ApiEntityType } from "../types/ApiEntityType";
	import PageHeader from "../components/PageHeader.svelte";
	import PageLoading from "../components/PageLoading.svelte";
	import EntityCard from "../components/EntityCard.svelte";

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
		<PageLoading />
	{:else}
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
			{#each types as type}
			<EntityCard
				icon={type.display_icon || 'lucide:box'}
				title={type.name}
				subtitle="Domain-specific entity classification and behavior definitions."
				onclick={() => router.navigate(`/entity/${ns}/${type.name}`)}
			/>
			{/each}
		</div>
	{/if}
</div>
