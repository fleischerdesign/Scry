<script lang="ts">
	import { onMount } from "svelte";
	import { api } from "../api";
	import { router } from "../router.svelte";
	import PageHeader from "../components/PageHeader.svelte";
	import PageLoading from "../components/PageLoading.svelte";
	import EmptyState from "../components/EmptyState.svelte";
	import EntityCard from "../components/EntityCard.svelte";
	import type { ApiNamespace } from "../types/ApiNamespace";

	let namespaces = $state<ApiNamespace[]>([]);
	let loading = $state(true);

	$effect(() => {
		router.title = "Explorer";
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
		subtitle="Explore entities across all namespaces and domains."
	/>

	{#if loading}
		<PageLoading />
	{:else}
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
		{#each namespaces as ns}
			<EntityCard
				icon={ns.display_icon || 'lucide:database'}
				title={ns.name}
				subtitle="Semantic domain containing specialized entity definitions and relationships."
				onclick={() => router.navigate(`/entity/${ns.name}`)}
			/>
		{:else}
			<EmptyState icon="lucide:layers" title="The graph is currently empty" description="Start ingesting data to populate the knowledge graph." />
			{/each}
		</div>
	{/if}
</div>
