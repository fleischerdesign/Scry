<script lang="ts">
	import { onMount } from "svelte";
	import { Chart, registerables } from "chart.js";
	import Card from "../components/Card.svelte";
	import { api } from "../api";
	import { ui } from "../ui.svelte";
	import { dashboards } from "../state/dashboards.svelte";
	import { router } from "../router.svelte";

	Chart.register(...registerables);

	let { onRefresh } = $props();

	$effect(() => {
		router.title = "Explorer";
	});

	let explorerCanvas = $state<HTMLCanvasElement>();
	let explorerChart: Chart | undefined;

	let selectedType = $state<string>("");
	let timeframe = $state<number>(7);
	let explorerData = $state<any[]>([]);
	let loading = $state(false);
	let catalog = $state<Record<string, any>>({});

	// Modal / Pin State
	let selectedDashboardId = $state("");
	let selectedWidth = $state(2);
	let pinning = $state(false);

	async function loadCatalog() {
		try {
			catalog = await api.getCatalog();
		} catch (e) {
			console.error(e);
		}
	}

	$effect(() => {
		if (dashboards.items.length > 0 && !selectedDashboardId) {
			selectedDashboardId = dashboards.items[0].id;
		}
	});

	onMount(loadCatalog);

	async function loadExplorerData() {
		if (!selectedType) return;
		loading = true;
		try {
			if (
				selectedType.includes("temperature") ||
				selectedType.includes("count") ||
				selectedType.includes("level")
			) {
				explorerData = await api.getSemanticSeries(selectedType, timeframe);
			} else {
				explorerData = await api.getSemanticTop(selectedType, 10, timeframe);
			}
		} catch (e) {
			console.error(e);
		} finally {
			loading = false;
		}
	}

	async function addToDashboard() {
		if (!selectedType || !selectedDashboardId) return;
		pinning = true;
		try {
			const isSeries = explorerData[0]?.label !== undefined;
			const widget = {
				type: isSeries ? "semantic_series" : "semantic_top",
				title: `${selectedType.split(".").pop()?.toUpperCase()} Trend`,
				width_span: selectedWidth,
				config: {
					semantic_type: selectedType,
					days: timeframe,
				},
			};
			await api.addWidget(selectedDashboardId, widget);
			ui.notify(
				"Widget Added",
				`Pinned ${selectedType} to dashboard`,
				"success",
			);
			onRefresh();
		} catch (e) {
			ui.notify("Failed to pin", "Check logs for details", "error");
		} finally {
			pinning = false;
		}
	}

	$effect(() => {
		if (selectedType || timeframe) loadExplorerData();
	});

	$effect(() => {
		if (explorerCanvas && explorerData.length > 0) {
			if (explorerChart) explorerChart.destroy();
			const isSeries = explorerData[0].label !== undefined;
			const labels = explorerData.map((d) => d.label || d.key);
			const values = explorerData.map((d) =>
				d.value !== undefined ? d.value : d.count,
			);

			explorerChart = new Chart(explorerCanvas, {
				type: isSeries ? "line" : "bar",
				data: {
					labels,
					datasets: [
						{
							data: values,
							backgroundColor: isSeries
								? "rgba(59, 130, 246, 0.1)"
								: "rgba(59, 130, 246, 0.4)",
							borderColor: "rgb(59, 130, 246)",
							borderWidth: 2,
							fill: isSeries,
							tension: 0.4,
							pointRadius: isSeries ? 4 : 0,
						},
					],
				},
				options: {
					responsive: true,
					maintainAspectRatio: false,
					scales: {
						y: { beginAtZero: true, grid: { color: "rgba(128,128,128,0.05)" } },
						x: { grid: { display: false } },
					},
					plugins: { legend: { display: false } },
				},
			});
		}
		return () => {
			if (explorerChart) explorerChart.destroy();
		};
	});
</script>

<div class="space-y-10 animate-in fade-in duration-500 w-full">
	<div class="flex items-center justify-between">
		<div class="join border border-base-300">
			{#each [1, 7, 30, 90] as d}
				<button
					class="btn btn-xs join-item font-mono {timeframe === d
						? 'btn-active'
						: 'btn-ghost opacity-50'}"
					onclick={() => (timeframe = d)}>{d}D</button
				>
			{/each}
		</div>
	</div>

	<div class="grid grid-cols-1 lg:grid-cols-4 gap-8">
		<aside class="lg:col-span-1 space-y-6">
			<div class="card bg-base-100 border border-base-300 shadow-sm">
				<div class="card-body p-4">
					<span
						class="text-[10px] font-black uppercase opacity-30 mb-4 tracking-widest text-center"
						>Available Metrics</span
					>
					<div class="flex flex-col gap-1">
						{#each Object.keys(catalog) as type}
							<button
								class="btn btn-ghost btn-sm justify-start {selectedType === type
									? 'btn-active bg-info/10 text-info'
									: 'opacity-60'}"
								onclick={() => (selectedType = type)}
							>
								<div
									class="w-1.5 h-1.5 rounded-full {selectedType === type
										? 'bg-info'
										: 'bg-base-300'} mr-2"
								></div>
								{type}
							</button>
						{/each}
					</div>
				</div>
			</div>

			{#if selectedType}
				<div
					class="card bg-base-100 border border-base-300 shadow-sm animate-in slide-in-from-left-4"
				>
					<div class="card-body p-4 space-y-4">
						<span
							class="text-[10px] font-black uppercase opacity-30 tracking-widest text-center"
							>Pin to Dashboard</span
						>

						<div class="form-control">
							<label class="label py-1" for="dash-select"
								><span class="label-text text-[10px] opacity-50">DASHBOARD</span
								></label
							>
							<select
								id="dash-select"
								bind:value={selectedDashboardId}
								class="select select-bordered select-xs font-mono"
							>
								{#each dashboards as dash}
									<option value={dash.id}>{dash.name}</option>
								{/each}
							</select>
						</div>

						<div class="form-control">
							<label class="label py-1" for="width-select"
								><span class="label-text text-[10px] opacity-50"
									>WIDTH SPAN</span
								></label
							>
							<select
								id="width-select"
								bind:value={selectedWidth}
								class="select select-bordered select-xs font-mono"
							>
								<option value={1}>Compact (1/4)</option>
								<option value={2}>Half (2/4)</option>
								<option value={4}>Full (4/4)</option>
							</select>
						</div>

						<button
							class="btn btn-info btn-xs w-full font-black tracking-tighter"
							onclick={addToDashboard}
							disabled={pinning}
						>
							{#if pinning}<span class="loading loading-spinner loading-xs"
								></span>{/if}
							ADD_TO_DASHBOARD
						</button>
					</div>
				</div>
			{/if}
		</aside>

		<div class="lg:col-span-3">
			<Card
				title={selectedType || "Select a stream"}
				subtitle={`TIME_SERIES / ${timeframe} DAYS`}
			>
				{#if loading}
					<div class="flex justify-center py-32">
						<span
							class="loading loading-infinity loading-lg text-info opacity-40"
						></span>
					</div>
				{:else if selectedType}
					<div class="h-[500px] w-full p-4">
						<canvas bind:this={explorerCanvas}></canvas>
					</div>
				{:else}
					<div
						class="flex flex-col items-center justify-center py-32 opacity-10"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="h-20 w-20 mb-6"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
							><path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="1"
								d="M14 10l-2 1m0 0l-2-1m2 1v2.5M20 7l-2 1m2-1l-2-1m2 1v2.5M14 4l-2-1-2 1M4 7l2-1M4 7l2 1M4 7v2.5M12 21l-2-1m2 1l2-1m-2 1V18.5M6 18l-2-1m2 1l2-1m-2 1V15.5M18 18l2-1m-2 1l-2-1m2 1V15.5"
							/></svg
						>
						<p class="font-mono text-xs uppercase tracking-[0.3em] font-black">
							Select a semantic node to visualize
						</p>
					</div>
				{/if}
			</Card>
		</div>
	</div>
</div>
