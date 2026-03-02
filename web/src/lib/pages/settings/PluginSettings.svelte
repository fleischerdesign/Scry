<script lang="ts">
	import { api } from "../../api";
	import ConfigField from "../../components/ConfigField.svelte";
	import { plugins } from "../../state/plugins.svelte";
	import { router } from "../../router.svelte";

	// Initialisiere pluginConfigs als reaktives Objekt
	let pluginConfigs = $state<Record<string, any>>({});
	let saving = $state(false);
	let successMessage = $state("");

	// Initialisiere die Config-Struktur sofort, wenn Plugins vorhanden sind
	$effect(() => {
		router.title = "Plugins";
		const newConfigs = { ...pluginConfigs };
		let changed = false;
		plugins.items.forEach((p) => {
			if (!newConfigs[p.id]) {
				newConfigs[p.id] = {};
				changed = true;
			}
		});
		if (changed) {
			pluginConfigs = newConfigs;
		}
	});

	async function savePluginConfig(pluginId: string) {
		saving = true;
		successMessage = "";
		try {
			await api.updatePluginConfig(pluginId, pluginConfigs[pluginId]);
			successMessage = `${pluginId} updated`;
			setTimeout(() => (successMessage = ""), 3000);
		} catch (e) {
			console.error(`Failed to save config for ${pluginId}`, e);
		} finally {
			saving = false;
		}
	}
</script>

<div class="space-y-4 animate-in slide-in-from-right-4 duration-300">
	{#if successMessage}
		<div
			class="badge badge-success font-mono text-[10px] py-3 px-4 animate-bounce fixed top-24 right-10 z-50"
		>
			{successMessage}
		</div>
	{/if}

	{#each plugins.items as plugin}
		<div
			class="collapse collapse-arrow bg-base-200 rounded-3xl border border-base-300 overflow-hidden"
		>
			<input type="checkbox" />
			<div class="collapse-title flex items-center gap-4 py-4">
				<div
					class="w-8 h-8 rounded-xl bg-base-300 flex items-center justify-center text-xs font-bold font-mono"
				>
					{plugin.name.charAt(0)}
				</div>
				<div>
					<h3 class="font-bold text-sm uppercase tracking-tight">
						{plugin.name}
					</h3>
					<p class="text-[10px] opacity-40 font-mono italic">
						{plugin.id} v{plugin.version}
					</p>
				</div>
			</div>
			<div class="collapse-content space-y-4 px-6 pb-6">
				<p class="text-xs opacity-60 leading-relaxed">{plugin.description}</p>

				<div class="flex flex-wrap gap-2">
					{#each plugin.capabilities as cap}
						<div class="badge badge-outline badge-xs font-mono opacity-50">
							{cap}
						</div>
					{/each}
				</div>

				{#if plugin.capabilities.includes("config")}
					<div class="divider opacity-10"></div>
					<div class="space-y-4">
						<h4
							class="text-[10px] font-black uppercase tracking-widest opacity-40"
						>
							Configuration
						</h4>

						{#if plugin.config_schema && pluginConfigs[plugin.id]}
							{@const schema = JSON.parse(plugin.config_schema)}
							<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
								{#each Object.entries(schema.properties || {}) as [key, prop]}
									<ConfigField
										{key}
										schema={prop}
										bind:value={pluginConfigs[plugin.id][key]}
									/>
								{/each}
							</div>
						{:else if !plugin.config_schema}
							<p class="text-[10px] italic opacity-30">
								No configuration schema defined by this node.
							</p>
						{:else}
							<span class="loading loading-dots loading-xs opacity-20"></span>
						{/if}

						<button
							class="btn btn-secondary btn-xs font-mono"
							onclick={() => savePluginConfig(plugin.id)}
							disabled={saving}
						>
							UPDATE_NODE
						</button>
					</div>
				{/if}
			</div>
		</div>
	{/each}
</div>
