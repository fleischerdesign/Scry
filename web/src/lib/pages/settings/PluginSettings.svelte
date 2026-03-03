<script lang="ts">
	import { onMount } from "svelte";
	import { api } from "../../api";
	import ConfigField from "../../components/ConfigField.svelte";
	import { plugins } from "../../state/plugins.svelte";
	import { router } from "../../router.svelte";

	let pluginConfigs = $state<Record<string, any>>({});
	let saving = $state(false);
	let successMessage = $state("");

	$effect(() => {
		router.title = "Extensions";
	});

	async function loadAllConfigs() {
		for (const p of plugins.items) {
			try {
				const cfg = await api.request(`/system/plugins/${p.id}/config`);
				pluginConfigs[p.id] = cfg;
			} catch (e) {
				console.error(`Failed to load config for ${p.id}`, e);
				pluginConfigs[p.id] = {};
			}
		}
	}

	onMount(loadAllConfigs);

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

	function getAliases(config: Record<string, any>) {
		return Object.entries(config || {}).filter(([k]) => k.startsWith('alias:'));
	}
</script>

<div class="space-y-4 animate-in slide-in-from-right-4 duration-300">
	{#if successMessage}
		<div
			class="badge badge-success font-mono text-[10px] py-3 px-4 animate-bounce fixed top-24 right-10 z-50 shadow-lg"
		>
			{successMessage}
		</div>
	{/if}

	{#each plugins.items as plugin}
		{@const aliases = getAliases(pluginConfigs[plugin.id])}

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
					<div class="flex items-center gap-2">
						<h3 class="font-bold text-sm uppercase tracking-tight">
							{plugin.name}
						</h3>
						<div class="flex gap-1">
							{#each plugin.roles as role}
								<div class="badge badge-neutral border-none text-[7px] font-black h-4 px-1.5 opacity-40">{role}</div>
							{/each}
							{#if aliases.length > 0}
								<div class="badge badge-primary border-none text-[7px] font-black h-4 px-1.5">MAPPED</div>
							{/if}
						</div>
					</div>
					<p class="text-[10px] opacity-40 font-mono italic">
						{plugin.id} v{plugin.version}
					</p>
				</div>
			</div>
			<div class="collapse-content space-y-6 px-6 pb-6 border-t border-base-300/50 pt-6">
				<p class="text-xs opacity-60 leading-relaxed">{plugin.description}</p>

				<div class="flex flex-wrap gap-2">
					{#each plugin.capabilities as cap}
						<div class="badge badge-outline badge-xs font-mono opacity-50 uppercase">
							{cap}
						</div>
					{/each}
				</div>

				<div class="grid grid-cols-1 md:grid-cols-2 gap-8">
					<!-- Semantic Wiring Section -->
					{#if aliases.length > 0}
						<div class="space-y-4 bg-primary/5 p-4 rounded-2xl border border-primary/10">
							<div class="flex items-center gap-2">
								<h4 class="text-[10px] font-black uppercase tracking-widest text-primary">Semantic Wiring</h4>
							</div>
							
							<div class="grid grid-cols-1 gap-4">
								{#each aliases as [key, value]}
									<div class="form-control w-full">
										<label class="label py-1" for={`${plugin.id}-${key}`}>
											<span class="label-text text-[9px] font-bold uppercase opacity-40">{key.replace('alias:', '')} slot</span>
										</label>
										<input
											type="text"
											id={`${plugin.id}-${key}`}
											bind:value={pluginConfigs[plugin.id][key]}
											placeholder="ns/type/id"
											class="input input-bordered input-sm font-mono text-[10px] flex-1 bg-base-100"
										/>
									</div>
								{/each}
							</div>
						</div>
					{/if}

					<!-- Technical Configuration -->
					{#if plugin.capabilities.includes("config")}
						<div class="space-y-4">
							<h4
								class="text-[10px] font-black uppercase tracking-widest opacity-40"
							>
								Node parameters
							</h4>

							{#if plugin.config_schema && pluginConfigs[plugin.id]}
								{@const schema = JSON.parse(plugin.config_schema)}
								<div class="grid grid-cols-1 gap-6">
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
									No custom parameters defined.
								</p>
							{:else}
								<span class="loading loading-dots loading-xs opacity-20"></span>
							{/if}
						</div>
					{/if}
				</div>

				<div class="pt-4 border-t border-base-300/30">
					<button
						class="btn btn-secondary btn-xs font-mono"
						onclick={() => savePluginConfig(plugin.id)}
						disabled={saving}
					>
						SAVE_CHANGES
					</button>
				</div>
			</div>
		</div>
	{/each}
</div>
