<script lang="ts">
	import { onMount } from "svelte";
	import { api } from "../api";
	import Card from "../components/Card.svelte";
	import { ui } from "../ui.svelte";
	import { router } from "../router.svelte";

	let discoveries = $state<any[]>([]);
	let loading = $state(true);
	let discovering = $state(false);

	$effect(() => {
		router.title = "Insights";
	});

	async function loadDiscoveries() {
		loading = true;
		try {
			discoveries = await api.getAnalytics("discoveries");
		} catch (e) {
			console.error("Failed to load discoveries", e);
		} finally {
			loading = false;
		}
	}

	async function runDiscovery() {
		discovering = true;
		try {
			const res = await api.getAnalytics("discover", { method: "POST" });
			ui.notify(
				`Discovery complete: Found ${res.new_discoveries} patterns`,
				"success",
			);
			await loadDiscoveries();
		} catch (e) {
			ui.notify("Discovery engine failed", "error");
		} finally {
			discovering = false;
		}
	}

	onMount(loadDiscoveries);

	function formatStrength(insights: string) {
		try {
			const data = JSON.parse(insights);
			const percent = Math.abs(data.strength * 100).toFixed(0);
			return { percent, type: data.strength > 0 ? "positive" : "negative" };
		} catch {
			return { percent: "0", type: "neutral" };
		}
	}
</script>

<div
	class="space-y-12 animate-in slide-in-from-bottom-4 duration-700 w-full pb-20"
>
	<div class="flex items-center justify-between pb-6">
		<button
			class="btn btn-primary gap-2"
			onclick={runDiscovery}
			disabled={discovering}
		>
			{#if discovering}
				<span class="loading loading-spinner loading-xs"></span>
				ANALYZING_LIFE_STREAMS...
			{:else}
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-3 w-3"
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
				Run Engine
			{/if}
		</button>
	</div>

	{#if loading}
		<div class="flex flex-col items-center justify-center py-32 opacity-20">
			<span class="loading loading-ring loading-lg mb-4"></span>
			<p class="text-[10px] font-black uppercase tracking-widest">
				Scanning knowledge graph...
			</p>
		</div>
	{:else if discoveries.length === 0}
		<div class="card bg-base-200 border-2 border-dashed border-base-300">
			<div class="card-body items-center text-center py-20">
				<div
					class="w-16 h-16 bg-base-300 rounded-full flex items-center justify-center mb-6 opacity-50"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-8 w-8"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						><path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z"
						/></svg
					>
				</div>
				<h2 class="card-title font-black uppercase tracking-tighter">
					No Insights Found Yet
				</h2>
				<p class="text-xs opacity-50 max-w-sm mb-8">
					The discovery engine needs more data or a manual trigger to find
					correlations between your life streams.
				</p>
				<button class="btn btn-primary btn-sm px-8" onclick={runDiscovery}
					>Start First Analysis</button
				>
			</div>
		</div>
	{:else}
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
			{#each discoveries as discovery}
				{@const stats = formatStrength(discovery.insights)}
				<div
					class="card bg-base-100 border border-base-300 shadow-xl hover:border-primary/50 transition-all group overflow-hidden"
				>
					<div class="card-body p-6">
						<div class="flex justify-between items-start mb-4">
							<div class="flex flex-col">
								<span
									class="text-[8px] font-black uppercase opacity-30 tracking-[0.2em] mb-1"
									>Observation_</span
								>
								<div
									class="badge badge-ghost badge-xs font-mono text-[8px] opacity-50"
								>
									PEARSON_CORR
								</div>
							</div>
							<div class="text-right">
								<div
									class="text-2xl font-black italic tracking-tighter"
									class:text-success={stats.type === "positive"}
									class:text-secondary={stats.type === "negative"}
								>
									{stats.percent}%
								</div>
								<span
									class="text-[8px] font-bold uppercase opacity-30 tracking-widest"
									>{stats.type} Match</span
								>
							</div>
						</div>

						<h3
							class="font-black text-lg leading-tight mb-4 group-hover:text-primary transition-colors italic"
						>
							There is a strong connection between <span class="text-secondary"
								>{discovery.source.split(".").pop()}</span
							>
							and
							<span class="text-primary"
								>{discovery.target.split(".").pop()}</span
							>.
						</h3>

						<div
							class="flex items-center gap-2 mt-auto pt-4 border-t border-base-300/50"
						>
							<div class="flex -space-x-2">
								<div
									class="w-6 h-6 rounded-full bg-base-300 border-2 border-base-100 flex items-center justify-center text-[8px] font-bold"
								>
									A
								</div>
								<div
									class="w-6 h-6 rounded-full bg-primary/20 border-2 border-base-100 flex items-center justify-center text-[8px] font-bold text-primary"
								>
									B
								</div>
							</div>
							<span
								class="text-[9px] font-mono opacity-40 uppercase tracking-tighter"
								>Cross-Plugin Inference</span
							>
						</div>
					</div>

					<!-- Background Pattern -->
					<div
						class="absolute -bottom-4 -right-4 w-24 h-24 bg-primary/5 rounded-full blur-2xl group-hover:bg-primary/10 transition-colors"
					></div>
				</div>
			{/each}
		</div>
	{/if}

	<!-- Manual Lab Section -->
	<section class="mt-20 pt-12 space-y-8">
		<div class="flex flex-col gap-2">
			<h3 class="text-xs font-black uppercase tracking-[0.4em] opacity-30">
				The Discovery Lab
			</h3>
			<p class="text-[9px] opacity-40 uppercase tracking-widest font-bold">
				Experimental manual correlation & hypothesis testing
			</p>
		</div>

		<div
			class="grid grid-cols-1 md:grid-cols-2 gap-8 opacity-50 hover:opacity-100 transition-opacity grayscale hover:grayscale-0"
		>
			<div class="card bg-base-200 border border-base-300">
				<div class="card-body p-8 items-center text-center">
					<h4 class="font-black uppercase tracking-tighter">
						Manual Join Engine
					</h4>
					<p class="text-[10px] opacity-50 mb-6">
						Test specific categories against each other in real-time.
					</p>
					<button class="btn btn-ghost btn-xs border-base-300">Open Lab</button>
				</div>
			</div>
			<div class="card bg-base-200 border border-base-300">
				<div class="card-body p-8 items-center text-center">
					<h4 class="font-black uppercase tracking-tighter">Semantic Stats</h4>
					<p class="text-[10px] opacity-50 mb-6">
						Detailed distribution analysis of individual traits.
					</p>
					<button class="btn btn-ghost btn-xs border-base-300"
						>View Metrics</button
					>
				</div>
			</div>
		</div>
	</section>
</div>
