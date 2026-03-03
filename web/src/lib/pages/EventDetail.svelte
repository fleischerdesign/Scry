<script lang="ts">
	import { onMount } from "svelte";
	import { api } from "../api";
	import { router } from "../router.svelte";
	import Card from "../components/Card.svelte";
	import type { Event } from "../types/Event";

	const { id } = router.getParams("/event/:id");
	let event = $state<Event | null>(null);
	let loading = $state(true);

	$effect(() => {
		router.title = "Event";
	});

	async function loadEvent() {
		loading = true;
		try {
			event = await api.getEvent(id);
		} catch (e) {
			console.error("Failed to load event", e);
		} finally {
			loading = false;
		}
	}

	onMount(loadEvent);
</script>

<div
	class="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500 w-full max-w-4xl pb-20"
>
	<!-- Header consistent with EntityDetail -->
	<div class="flex items-start gap-6 border-b border-base-300 pb-8">
		<button
			class="btn btn-ghost btn-sm btn-square mt-2"
			onclick={() => window.history.back()}
		>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				class="h-5 w-5"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
				><path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M15 19l-7-7 7-7"
				/></svg
			>
		</button>

		{#if !loading && event?.display_image}
			<div class="avatar">
				<div class="w-24 h-24 rounded-3xl shadow-2xl ring ring-primary/20 overflow-hidden bg-base-300">
					<img src={event.display_image} alt={event.display_title || ''} class="object-cover w-full h-full" />
				</div>
			</div>
		{:else if !loading}
			<div class="w-24 h-24 rounded-3xl bg-base-300 flex items-center justify-center text-3xl font-black opacity-20 uppercase">
				{(event?.display_title || "E").charAt(0)}
			</div>
		{/if}

		<div class="flex-1">
			<div class="badge badge-secondary badge-outline font-mono text-[9px] uppercase tracking-widest mb-2">Event / {event?.category.split('.').pop()}</div>
			<h2 class="text-4xl font-black tracking-tighter italic uppercase text-base-content">
				{event?.display_title || "EVENT_DETAIL"}
			</h2>
			{#if event?.display_subtitle}
				<p class="text-[10px] font-mono opacity-40 uppercase tracking-widest mt-2">
					{event.display_subtitle}
				</p>
			{/if}
		</div>
	</div>

	{#if loading}
		<div class="flex justify-center py-20">
			<span class="loading loading-ring loading-lg opacity-20"></span>
		</div>
	{:else if event}
		<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
			<!-- Main Content -->
			<div class="md:col-span-2 space-y-6">
				<Card title="Raw Payload" subtitle={event.category}>
					<pre
						class="bg-base-300 p-4 rounded-xl font-mono text-xs overflow-x-auto text-primary/80">
                        {JSON.stringify(event.payload, null, 4)}
                    </pre>
				</Card>

				{#if event.metadata}
					<Card title="Metadata" subtitle="ENRICHMENT_INFO">
						<div class="grid grid-cols-1 gap-2">
							{#each Object.entries(event.metadata) as [k, v]}
								<div
									class="flex justify-between items-center bg-base-200 p-3 rounded-xl border border-base-300/50"
								>
									<span class="text-[10px] font-black uppercase opacity-40"
										>{k}</span
									>
									<span class="font-mono text-xs">{v}</span>
								</div>
							{/each}
						</div>
					</Card>
				{/if}
			</div>

			<!-- Sidebar Info -->
			<div class="space-y-6">
				<Card title="Context" subtitle="TECHNICAL">
					<div class="space-y-4">
						<div>
							<p class="text-[9px] font-black uppercase opacity-30 mb-1">
								Event UUID
							</p>
							<p class="font-mono text-[10px] opacity-60 truncate">
								{id}
							</p>
						</div>
						<div>
							<p class="text-[9px] font-black uppercase opacity-30 mb-1">
								Timestamp
							</p>
							<p class="font-mono text-xs">
								{new Date(event.timestamp).toLocaleString()}
							</p>
						</div>
						<div>
							<p class="text-[9px] font-black uppercase opacity-30 mb-1">
								Source Node
							</p>
							<div class="badge badge-outline font-mono text-[10px] uppercase">
								{event.source}
							</div>
						</div>
					</div>
				</Card>

				{#if event.entities && event.entities.length > 0}
					<Card title="Linked Entities" subtitle="SEMANTIC_GRAPH">
						<div class="space-y-2">
							{#each event.entities as ent}
								<button
									onclick={() =>
										router.navigate(
											`/entity/${ent.namespace}/${ent.typ}/${ent.id}`,
										)}
									class="w-full flex items-center gap-3 p-3 bg-base-200 hover:bg-base-300 transition-all rounded-xl border border-base-300/50 group text-left"
								>
									<div
										class="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center text-primary"
									>
										<svg
											xmlns="http://www.w3.org/2000/svg"
											class="h-4 w-4"
											fill="none"
											viewBox="0 0 24 24"
											stroke="currentColor"
											><path
												stroke-linecap="round"
												stroke-linejoin="round"
												stroke-width="2"
												d="M13 10V3L4 14h7v7l9-11h-7z"
											/></svg
										>
									</div>
									<div class="flex-1 overflow-hidden">
										<span class="font-bold text-xs block truncate"
											>{ent.id}</span
										>
										<span class="text-[9px] opacity-40 uppercase truncate"
											>{ent.namespace}/{ent.typ}</span
										>
									</div>
									<svg
										xmlns="http://www.w3.org/2000/svg"
										class="h-3 w-3 opacity-0 group-hover:opacity-40 transition-opacity"
										fill="none"
										viewBox="0 0 24 24"
										stroke="currentColor"
										><path
											stroke-linecap="round"
											stroke-linejoin="round"
											stroke-width="2"
											d="M9 5l7 7-7 7"
										/></svg
									>
								</button>
							{/each}
						</div>
					</Card>
				{/if}
			</div>
		</div>
	{/if}
</div>
