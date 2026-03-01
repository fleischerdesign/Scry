<script lang="ts">
    import { onMount } from 'svelte';
    import { Chart, registerables } from 'chart.js';
    import { api } from '../api';
    import Card from './Card.svelte';

    Chart.register(...registerables);

    let { widget } = $props();
    let canvas = $state<HTMLCanvasElement>();
    let chart: Chart | undefined;
    let loading = $state(true);
    let data = $state<any[]>([]);
    let latestValue = $state<any>(null);

    async function loadWidgetData() {
        loading = true;
        try {
            const config = typeof widget.config === 'string' ? JSON.parse(widget.config) : widget.config;
            const { semantic_type, category, path, days = 7 } = config;

            if (widget.type === 'Trend' || widget.type === 'semantic_series') {
                data = await api.getSemanticSeries(semantic_type, days);
            } else if (widget.type === 'TopList' || widget.type === 'semantic_top') {
                data = await api.getSemanticTop(semantic_type, 10, days);
            } else if (widget.type === 'Metric' || widget.type === 'stat') {
                const latest = await api.getData(category, 1);
                if (latest && latest.length > 0) {
                    latestValue = path ? latest[0].event[path] : latest[0].event;
                }
            }
        } catch (e) {
            console.error("Widget Data Load Error", e);
        } finally {
            loading = false;
        }
    }

    function renderChart(node: HTMLCanvasElement, plotData: any[]) {
        if (plotData.length === 0) return;
        if (chart) chart.destroy();

        const labels = plotData.map(d => d.label || d.key);
        const values = plotData.map(d => d.value !== undefined ? d.value : d.count);

        chart = new Chart(node, {
            type: widget.type === 'Trend' || widget.type === 'semantic_series' ? 'line' : 'bar',
            data: {
                labels,
                datasets: [{
                    data: values,
                    backgroundColor: 'rgba(59, 130, 246, 0.1)',
                    borderColor: 'rgb(59, 130, 246)',
                    borderWidth: 2,
                    fill: true,
                    tension: 0.4,
                    pointRadius: 2
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: { beginAtZero: true, grid: { color: 'rgba(128,128,128,0.05)' }, ticks: { font: { size: 10 } } },
                    x: { grid: { display: false }, ticks: { display: widget.width_span > 1, font: { size: 10 } } }
                },
                plugins: { legend: { display: false } }
            }
        });
    }

    onMount(loadWidgetData);
    $effect(() => { if (canvas && data.length > 0) renderChart(canvas, data); });
</script>

<div class="h-full w-full" style="grid-column: span {widget.width_span}">
    <Card title={widget.title} subtitle={widget.type.toUpperCase()}>
        <div class="h-40 w-full relative flex flex-col justify-center">
            {#if loading}
                <div class="absolute inset-0 flex justify-center items-center opacity-20">
                    <span class="loading loading-spinner loading-md"></span>
                </div>
            {:else if widget.type === 'Metric' || widget.type === 'stat'}
                <div class="flex flex-col items-center">
                    <span class="text-5xl font-black tracking-tighter text-primary">
                        {latestValue !== null ? latestValue : '---'}
                    </span>
                    {#if (widget.config?.unit)}
                        <span class="text-xs font-mono opacity-30 mt-1 uppercase tracking-widest">{widget.config.unit}</span>
                    {/if}
                </div>
            {:else if (widget.type === 'TopList' || widget.type === 'semantic_top') && data.length > 0}
                <div class="space-y-2 overflow-y-auto max-h-full px-2">
                    {#each data.slice(0, 5) as item}
                        <div class="flex justify-between items-center text-[11px]">
                            <span class="font-bold truncate max-w-[70%]">{item.key}</span>
                            <div class="flex items-center gap-2">
                                <div class="w-12 h-1 bg-base-300 rounded-full overflow-hidden">
                                    <div class="bg-secondary h-full" style="width: {(item.count / data[0].count) * 100}%"></div>
                                </div>
                                <span class="opacity-40 font-mono">{item.count}</span>
                            </div>
                        </div>
                    {/each}
                </div>
            {:else if data.length > 0}
                <canvas bind:this={canvas}></canvas>
            {:else}
                <div class="absolute inset-0 flex justify-center items-center opacity-10 italic text-[10px] uppercase font-black tracking-widest">
                    No data available
                </div>
            {/if}
        </div>
    </Card>
</div>
