<script lang="ts">
	import { onMount, untrack } from "svelte";
	import { Chart, registerables } from "chart.js";
	import { api } from "../api";
	import { ui } from "../ui.svelte";
	import { router } from "../router.svelte";
	import { getThemeCSS } from "../theme";
	import Icon from "@iconify/svelte";
	import PageHeader from "../components/PageHeader.svelte";
	import PageLoading from "../components/PageLoading.svelte";
	import EmptyState from "../components/EmptyState.svelte";

	Chart.register(...registerables);

	$effect(() => {
		router.title = "Lab";
	});

	// --- State ---
	interface Query {
		id: string;
		type: string;
		label: string;
		color: string;
		data: any[];
		visible: boolean;
	}

	let queries = $state<Query[]>([]);
	let timeframe = $state<number>(7);
	let interval = $state<string>("auto");
	let loading = $state(false);
	let catalog = $state<Record<string, any>>({});
	let normalize = $state(true);
	let showTable = $state(false);

	let canvas = $state<HTMLCanvasElement>();
	let chart: Chart | undefined;

	const COLORS = [
		"#3b82f6", // blue
		"#ef4444", // red
		"#10b981", // green
		"#f59e0b", // amber
		"#8b5cf6", // violet
		"#ec4899", // pink
	];

	const INTERVALS = [
		{ id: "auto", label: "Auto" },
		{ id: "1h", label: "1 Hour" },
		{ id: "1d", label: "1 Day" },
		{ id: "1w", label: "1 Week" },
	];

	// --- Logic ---
	async function loadCatalog() {
		try {
			catalog = await api.getCatalog();
		} catch (e) {
			console.error(e);
		}
	}

	async function addQuery(type: string) {
		if (queries.find((q) => q.type === type)) return;
		
		const id = Math.random().toString(36).substring(7);
		const color = COLORS[queries.length % COLORS.length];
		const newQuery: Query = {
			id,
			type,
			label: type.split(".").pop() || type,
			color,
			data: [],
			visible: true,
		};
		
		queries = [...queries, newQuery];
		await refreshQuery(id);
	}

	function removeQuery(id: string) {
		queries = queries.filter((q) => q.id !== id);
	}

	function getEffectiveInterval() {
		if (interval !== "auto") return interval;
		if (timeframe <= 2) return "1h";
		if (timeframe <= 31) return "1d";
		return "1w";
	}

	async function refreshQuery(id: string) {
		const q = queries.find((it) => it.id === id);
		if (!q) return;

		loading = true;
		try {
			const activeInterval = getEffectiveInterval();
			const data = await api.getSemanticSeries(q.type, timeframe, activeInterval);
			queries = queries.map((it) => (it.id === id ? { ...it, data } : it));
		} catch (e) {
			console.error(`Failed to load data for ${q.type}`, e);
			ui.notify("Query Failed", `Could not load ${q.type}`, "error");
		} finally {
			loading = false;
		}
	}

	async function refreshAll() {
		for (const q of queries) {
			await refreshQuery(q.id);
		}
	}

	// Normalization: Map all values to 0-100 range for visual comparison
	function getProcessedData(query: Query) {
		if (query.data.length === 0) return [];
		if (!normalize) return query.data.map(d => d.value);

		const values = query.data.map((d) => d.value);
		const min = Math.min(...values);
		const max = Math.max(...values);
		const range = max - min;

		if (range === 0) return values.map(() => 50); // Flat line in middle if no variance
		return values.map((v) => ((v - min) / range) * 100);
	}

	$effect(() => {
		if (timeframe || interval) {
			untrack(() => refreshAll());
		}
	});

	$effect(() => {
		if (canvas && queries.length > 0) {
			if (chart) chart.destroy();

			const gridColor = getThemeCSS("--color-base-300", "#d1d5db");
			const tickColor = getThemeCSS("--color-base-content", "#888");
			const tooltipBg = getThemeCSS("--color-base-300", "#1f2937");

			// Use the first query with data to determine the labels (X-axis)
			const firstWithData = queries.find(q => q.data.length > 0);
			const labels = firstWithData ? firstWithData.data.map(d => d.label) : [];

			chart = new Chart(canvas, {
				type: "line",
				data: {
					labels,
					datasets: queries.filter(q => q.visible).map((q) => ({
						label: q.label,
						data: getProcessedData(q),
						borderColor: q.color,
						backgroundColor: `${q.color}10`,
						borderWidth: 2,
						fill: true,
						tension: 0.4,
						pointRadius: 2,
					})),
				},
				options: {
					responsive: true,
					maintainAspectRatio: false,
					animation: { duration: 400 },
					interaction: { mode: 'index', intersect: false },
					scales: {
						y: { 
							beginAtZero: true, 
							grid: { color: gridColor + '14' },
							ticks: {
								color: tickColor + '66',
								font: { size: 10, family: 'monospace' },
								callback: (val) => normalize ? `${val}%` : val
							}
						},
						x: { 
							grid: { display: false },
							ticks: {
								color: tickColor + '66',
								font: { size: 10, family: 'monospace' },
								maxRotation: 0,
								autoSkip: true,
								maxTicksLimit: 12
							}
						},
					},
					plugins: { 
						legend: { display: false },
						tooltip: {
							backgroundColor: tooltipBg,
							padding: 12,
							cornerRadius: 12,
							titleFont: { size: 10, family: 'monospace' },
							bodyFont: { size: 12, family: 'monospace' },
							callbacks: {
								label: (context) => {
									const q = queries[context.datasetIndex];
									const rawValue = q.data[context.dataIndex]?.value;
									return `${q.label}: ${rawValue} ${normalize && context.parsed.y != null ? `(${context.parsed.y.toFixed(1)}%)` : ''}`;
								}
							}
						}
					},
				},
			});
		}
		return () => {
			if (chart) chart.destroy();
		};
	});

	onMount(loadCatalog);
</script>

<div class="flex flex-col h-full space-y-8 animate-in fade-in duration-500">
	<PageHeader 
		title="Laboratory" 
		subtitle="Multi-stream workbench for experimental hypothesis testing and pattern analysis."
	>
		{#snippet actions()}
			<div class="flex items-center gap-4 bg-base-200/50 p-1.5 rounded-2xl border border-base-300">
				<div class="flex items-center gap-1.5 px-2">
					<span class="text-xs font-bold opacity-60 tracking-wide mr-1">Timeframe</span>
					<div class="join border border-base-300/50 bg-base-100">
						{#each [1, 7, 30, 90, 365] as d}
							<button
								class="btn btn-xs join-item {timeframe === d ? 'btn-primary' : 'btn-ghost opacity-60'}"
								onclick={() => (timeframe = d)}
							>
								{d === 365 ? '1Y' : d + 'D'}
							</button>
						{/each}
					</div>
				</div>

				<div class="w-px h-6 bg-base-300 opacity-70"></div>

				<div class="flex items-center gap-1.5 px-2">
					<span class="text-xs font-bold opacity-60 tracking-wide mr-1">Resolution</span>
					<div class="join border border-base-300/50 bg-base-100">
						{#each INTERVALS as int}
							<button
								class="btn btn-xs join-item {interval === int.id ? 'btn-secondary' : 'btn-ghost opacity-60'}"
								onclick={() => (interval = int.id)}
							>
								{int.label.split(' ').pop()}
							</button>
						{/each}
					</div>
				</div>

				<div class="w-px h-6 bg-base-300 opacity-70"></div>

				<button 
					class="btn btn-xs btn-ghost gap-2 transition-all {normalize ? 'text-primary' : 'opacity-70 hover:opacity-100'}"
					onclick={() => normalize = !normalize}
				>
					<Icon icon={normalize ? "lucide:layers-2" : "lucide:layers"} class="w-3.5 h-3.5" />
					<span class="text-xs font-bold tracking-wide">Normalize</span>
				</button>
			</div>
		{/snippet}
	</PageHeader>

	<div class="grid grid-cols-1 xl:grid-cols-4 gap-8 flex-1">
		<!-- Query Editor Sidebar -->
		<aside class="xl:col-span-1 space-y-6">
			<div class="card bg-base-100 border border-base-300 shadow-sm overflow-hidden rounded-2xl">
				<div class="p-5 bg-base-200/50 border-b border-base-300 flex items-center justify-between">
					<span class="text-xs font-bold opacity-60 tracking-wide">Active Queries</span>
					<div class="badge badge-primary badge-xs font-mono">{queries.length}</div>
				</div>
				<div class="p-3 space-y-2 max-h-[400px] overflow-y-auto">
					{#each queries as q}
						<div class="flex items-center gap-3 p-3 bg-base-200/50 hover:bg-base-200 rounded-2xl group transition-all border border-transparent hover:border-base-300/50">
							<div class="w-1.5 h-10 rounded-full shrink-0" style="background-color: {q.color}"></div>
							<div class="flex-1 min-w-0">
								<div class="text-xs font-bold truncate opacity-80 tracking-tighter">{q.label}</div>
								<div class="text-xs opacity-70 truncate font-mono">{q.type}</div>
							</div>
							<button 
								class="btn btn-ghost btn-xs btn-square opacity-0 group-hover:opacity-100 transition-opacity"
								onclick={() => removeQuery(q.id)}
								aria-label="Remove query"
							>
								<Icon icon="lucide:trash-2" class="w-3.5 h-3.5 text-error" />
							</button>
						</div>
					{:else}
					<div class="py-16 text-center opacity-60">
						<Icon icon="lucide:flask-conical" class="w-10 h-10 mx-auto mb-3" />
						<p class="text-xs font-bold tracking-wide">No queries defined</p>
						</div>
					{/each}
				</div>
			</div>

			<div class="card bg-base-100 border border-base-300 shadow-sm overflow-hidden rounded-2xl">
				<div class="p-5 bg-base-200/50 border-b border-base-300">
					<span class="text-xs font-bold opacity-60 tracking-wide">Data Catalog</span>
				</div>
				<div class="p-3 space-y-1 max-h-[300px] overflow-y-auto">
					{#each Object.keys(catalog) as type}
						<button
							class="btn btn-ghost btn-sm btn-block justify-start text-xs font-bold opacity-60 hover:opacity-100 hover:text-primary rounded-xl"
							onclick={() => addQuery(type)}
							disabled={queries.some(q => q.type === type)}
						>
							<Icon icon="lucide:plus" class="w-3 h-3 mr-2" />
							{type}
						</button>
					{/each}
				</div>
			</div>
		</aside>

		<!-- Main Result Area -->
		<div class="xl:col-span-3 space-y-6 flex flex-col min-h-0">
			<!-- Chart Area -->
			<div class="bg-base-100 border border-base-300 rounded-3xl p-10 shadow-sm relative flex-1 min-h-[500px] overflow-hidden">
				<!-- Subtle Grid Decoration -->
				<div class="absolute inset-0 opacity-[0.03] pointer-events-none" style="background-image: linear-gradient(#ccc 1px, transparent 1px), linear-gradient(90deg, #ccc 1px, transparent 1px); background-size: 50px 50px;"></div>
				
				{#if loading && queries.length === 0}
					<PageLoading label="Initializing engine..." />
				{:else if queries.length === 0}
					<EmptyState icon="lucide:search" title="No data streams selected" description="Select data streams from the sidebar to start analysis." />
				{:else}
					<div class="h-full w-full relative z-10">
						<canvas bind:this={canvas}></canvas>
					</div>
				{/if}

				{#if loading && queries.length > 0}
					<div class="absolute top-6 right-8">
						<span class="loading loading-spinner loading-xs opacity-60"></span>
					</div>
				{/if}
			</div>

			<!-- Data Inspector Switch -->
			<div class="flex justify-center">
				<button 
					class="btn btn-ghost btn-xs gap-3 tracking-wide opacity-70 hover:opacity-100 transition-all bg-base-200/50 px-6 rounded-full"
					onclick={() => showTable = !showTable}
				>
					<Icon icon={showTable ? "lucide:chevron-up" : "lucide:chevron-down"} class="w-3 h-3" />
					{showTable ? 'Hide' : 'Show'} Raw Data Inspector
				</button>
			</div>

			<!-- Data Table -->
			{#if showTable && queries.length > 0}
				<div class="card bg-base-100 border border-base-300 shadow-xl overflow-hidden animate-in slide-in-from-bottom-6 rounded-2xl">
					<div class="overflow-x-auto">
						<table class="table table-xs font-mono">
							<thead class="bg-base-200">
								<tr>
									<th class="p-4">TIMESTAMP</th>
									{#each queries as q}
										<th class="p-4" style="color: {q.color}">{q.label.toUpperCase()}</th>
									{/each}
								</tr>
							</thead>
							<tbody>
								{#each queries[0].data as _, rowIndex}
									<tr class="hover:bg-base-200/50 transition-colors border-base-300/50">
										<td class="p-4 opacity-70">{queries[0].data[rowIndex].label}</td>
										{#each queries as q}
											<td class="p-4 font-bold">
												{q.data[rowIndex]?.value ?? 'N/A'}
											</td>
										{/each}
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				</div>
			{/if}
		</div>
	</div>
</div>

<style>
	canvas {
		filter: drop-shadow(0 0 15px color-mix(in oklch, var(--color-primary) 8%, transparent));
	}
	
	/* Custom scrollbar for catalogs */
	.overflow-y-auto::-webkit-scrollbar {
		width: 4px;
	}
	.overflow-y-auto::-webkit-scrollbar-track {
		background: transparent;
	}
	.overflow-y-auto::-webkit-scrollbar-thumb {
		background: color-mix(in oklch, var(--color-base-content) 10%, transparent);
		border-radius: 10px;
	}
</style>
