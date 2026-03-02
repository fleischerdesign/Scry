<script lang="ts">
	import { plugins } from "../../state/plugins.svelte";
	import { router } from "../../router.svelte";

	const enrichers = $derived(plugins.enrichers);

	$effect(() => {
		router.title = "Enricher";
	});
</script>

<div class="space-y-4 animate-in slide-in-from-right-4 duration-300">
	<div
		class="alert bg-primary/5 border border-primary/10 text-xs leading-relaxed rounded-3xl p-6"
	>
		<svg
			xmlns="http://www.w3.org/2000/svg"
			fill="none"
			viewBox="0 0 24 24"
			class="stroke-primary shrink-0 w-6 h-6"
			><path
				stroke-linecap="round"
				stroke-linejoin="round"
				stroke-width="2"
				d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
			></path></svg
		>
		<span
			>Enricher nodes automatically augment your data with semantic traits like
			photos, biographies, or location data.</span
		>
	</div>

	{#each enrichers as enricher}
		<div class="bg-base-200 p-6 rounded-3xl border border-base-300 space-y-4">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-3">
					<div
						class="w-8 h-8 rounded-xl bg-accent/20 text-accent flex items-center justify-center"
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
					<h3 class="font-bold text-sm uppercase tracking-tight">
						{enricher.name}
					</h3>
				</div>
				<div
					class="badge badge-accent badge-outline badge-xs font-mono uppercase opacity-50 italic"
				>
					Active
				</div>
			</div>

			<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
				{#if enricher.provided_traits && enricher.provided_traits.length > 0}
					<div class="space-y-2">
						<h4
							class="text-[9px] font-black uppercase tracking-widest opacity-30"
						>
							Active Knowledge Traits
						</h4>
						<div class="flex flex-wrap gap-2">
							{#each enricher.provided_traits as trait}
								<div
									class="badge badge-ghost badge-sm gap-2 font-mono border border-base-300"
								>
									<span class="opacity-40"
										>{trait.entity_namespace}/{trait.entity_type}</span
									>
									<span class="text-primary">→</span>
									<span class="font-bold"
										>{trait.trait_id.split("/").pop()}</span
									>
								</div>
							{/each}
						</div>
					</div>
				{/if}

				{#if enricher.exports && enricher.exports.length > 0}
					<div class="space-y-2">
						<h4
							class="text-[9px] font-black uppercase tracking-widest opacity-30"
						>
							Inline Event Enrichment
						</h4>
						<div class="flex flex-wrap gap-2">
							{#each enricher.exports as exp}
								<div
									class="badge badge-secondary/10 text-secondary badge-sm gap-2 font-mono border border-secondary/20"
								>
									<span class="opacity-60">{exp.category}</span>
									<span class="opacity-30">::</span>
									<span class="font-bold">{exp.semantic_type}</span>
								</div>
							{/each}
						</div>
					</div>
				{/if}

				{#if (!enricher.provided_traits || enricher.provided_traits.length === 0) && (!enricher.exports || enricher.exports.length === 0)}
					<p class="text-[10px] italic opacity-30">
						Semantic processor without explicit trait declarations.
					</p>
				{/if}
			</div>
		</div>
	{/each}
</div>
