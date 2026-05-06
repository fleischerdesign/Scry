<script lang="ts">
	import { onMount } from "svelte";
	import { api } from "../api";
	import Card from "../components/Card.svelte";
	import { ui } from "../ui.svelte";
	import { router } from "../router.svelte";
	import PageHeader from "../components/PageHeader.svelte";
	import Icon from "@iconify/svelte";

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
	<PageHeader 
		title="Insights" 
		subtitle="Automated pattern recognition and cross-plugin correlations discovered from your data."
	>
		{#snippet actions()}
			<button
				class="btn btn-sm btn-ghost gap-2 border border-base-300 rounded-xl font-bold opacity-60 hover:opacity-100 transition-all"
				onclick={runDiscovery}
				disabled={discovering}
			>
				{#if discovering}
					<span class="loading loading-spinner loading-xs"></span>
					ANALYZING...
				{:else}
					<Icon icon="lucide:zap" class="w-4 h-4 text-primary" />
					Run Engine
				{/if}
			</button>
		{/snippet}
	</PageHeader>

	{#if loading}
		<div class="flex flex-col items-center justify-center py-32 opacity-60">
			<span class="loading loading-spinner loading-lg mb-4"></span>
			<p class="text-xs font-bold tracking-wide">
				Loading discoveries...
			</p>
		</div>
	{:else if discoveries.length === 0}
		<div class="card bg-base-200 border-2 border-dashed border-base-300">
			<div class="card-body items-center text-center py-20">
				<div
					class="w-16 h-16 bg-base-300 rounded-full flex items-center justify-center mb-6 opacity-70"
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
				<h2 class="card-title font-black tracking-tighter">
					No Insights Found Yet
				</h2>
				<p class="text-xs opacity-70 max-w-sm mb-8">
					The system needs more data or a manual trigger to find
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
									class="text-xs font-bold opacity-60 tracking-wide mb-1"
									>Observation</span
								>
								<div
									class="badge badge-ghost badge-xs font-mono text-xs opacity-70"
								>
									Correlation
								</div>
							</div>
							<div class="text-right">
								<div
									class="text-2xl font-bold tracking-tighter"
									class:text-success={stats.type === "positive"}
									class:text-secondary={stats.type === "negative"}
								>
									{stats.percent}%
								</div>
								<span
									class="text-xs font-bold opacity-60 tracking-wide"
									>{stats.type} Match</span
								>
							</div>
						</div>

						<h3
							class="font-bold text-lg leading-tight mb-4 group-hover:text-primary transition-colors"
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
									class="w-6 h-6 rounded-full bg-base-300 border-2 border-base-100 flex items-center justify-center text-xs font-bold"
								>
									A
								</div>
								<div
									class="w-6 h-6 rounded-full bg-primary/20 border-2 border-base-100 flex items-center justify-center text-xs font-bold text-primary"
								>
									B
								</div>
							</div>
							<span
								class="text-xs opacity-70 tracking-tighter"
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
			<h3 class="text-xs font-black tracking-wide opacity-60">
				Manual Analysis
			</h3>
			<p class="text-xs opacity-70 tracking-wide font-bold">
				Experimental manual correlation and hypothesis testing
			</p>
		</div>

		<div
			class="grid grid-cols-1 md:grid-cols-2 gap-8 opacity-70 hover:opacity-100 transition-opacity grayscale hover:grayscale-0"
		>
			<div class="card bg-base-200 border border-base-300">
				<div class="card-body p-8 items-center text-center">
					<h4 class="font-black tracking-tighter">
						Manual Correlation
					</h4>
					<p class="text-xs opacity-70 mb-6">
						Test specific categories against each other in real-time.
					</p>
					<button class="btn btn-ghost btn-xs border-base-300">Open Lab</button>
				</div>
			</div>
			<div class="card bg-base-200 border border-base-300">
				<div class="card-body p-8 items-center text-center">
					<h4 class="font-black tracking-tighter">Semantic Stats</h4>
					<p class="text-xs opacity-70 mb-6">
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
