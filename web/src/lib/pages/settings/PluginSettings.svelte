<script lang="ts">
	import { router } from "../../router.svelte";
	import { createPluginsQuery } from "../../queries/plugins";
	import PageHeader from "../../components/PageHeader.svelte";
	import Icon from "@iconify/svelte";

	const pluginsQuery = createPluginsQuery();
</script>

<div class="space-y-8 animate-in fade-in duration-500 w-full max-w-4xl pb-20">
	<PageHeader 
		title="Extensions" 
		subtitle="Manage and configure WASM nodes, data sources, and semantic enrichers."
		onBack={() => router.navigate("/settings")}
	/>

	{#if pluginsQuery.isLoading}
		<div class="flex justify-center py-20 opacity-60">
			<span class="loading loading-spinner loading-lg text-primary"></span>
		</div>
	{:else if pluginsQuery.data}
		<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
			{#each pluginsQuery.data as plugin}
				<button 
					onclick={() => router.navigate(`/settings/plugins/${plugin.id}`)}
					class="card bg-base-100 border border-base-300 shadow-sm hover:border-primary transition-all group text-left rounded-[2rem] overflow-hidden"
				>
					<div class="card-body p-6">
						<div class="flex items-center justify-between mb-4">
							<div class="w-12 h-12 rounded-2xl bg-base-200 flex items-center justify-center group-hover:scale-110 transition-transform shadow-inner">
								<Icon icon="lucide:puzzle" class="w-6 h-6 opacity-70 group-hover:text-primary group-hover:opacity-100 transition-all" />
							</div>
							<div class="flex flex-col items-end">
								<span class="badge badge-ghost font-mono text-xs opacity-70">{plugin.version}</span>
								{#if plugin.capabilities.includes('oauth')}
									<div class="mt-1 flex items-center gap-1 text-xs font-bold text-primary tracking-wide">
										<Icon icon="lucide:key-round" class="w-2.5 h-2.5" />
										Auth Required
									</div>
								{/if}
							</div>
						</div>

						<h3 class="font-black text-lg tracking-tight mb-1 ">{plugin.name}</h3>
						<p class="text-xs opacity-70 line-clamp-2 leading-relaxed mb-6 h-8">
							{plugin.description || 'No description provided for this extension.'}
						</p>

						<div class="flex items-center justify-between pt-4 border-t border-base-300/50">
							<div class="flex items-center gap-1.5 text-xs font-bold text-success">
        <div class="w-1.5 h-1.5 rounded-full bg-success animate-pulse"></div>
        Active
       </div>
							<div class="flex items-center gap-1 text-primary opacity-0 group-hover:opacity-100 transition-opacity">
								<span class="text-xs font-bold tracking-wide">Configure</span>
								<Icon icon="lucide:chevron-right" class="w-3.5 h-3.5" />
							</div>
						</div>
					</div>
				</button>
			{/each}
		</div>
	{:else}
		<div class="card bg-base-200 border-2 border-dashed border-base-300 p-20 text-center rounded-[3rem]">
			<Icon icon="lucide:alert-circle" class="w-12 h-12 mx-auto mb-4 opacity-60" />
			<p class="text-xs tracking-wide opacity-70">No extensions detected</p>
		</div>
	{/if}
</div>
