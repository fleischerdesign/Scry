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

    async function loadWidgetData() {
        loading = true;
        try {
            const { semantic_type, days } = widget.config;
            if (widget.type === 'semantic_series') {
                data = await api.getSemanticSeries(semantic_type, days || 7);
            } else if (widget.type === 'semantic_top') {
                data = await api.getSemanticTop(semantic_type, 10, days);
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

        const isSeries = widget.type === 'semantic_series';
        const labels = plotData.map(d => d.label || d.key);
        const values = plotData.map(d => d.value !== undefined ? d.value : d.count);

        chart = new Chart(node, {
            type: isSeries ? 'line' : 'bar',
            data: {
                labels,
                datasets: [{
                    data: values,
                    backgroundColor: isSeries ? 'rgba(59, 130, 246, 0.1)' : 'rgba(147, 51, 234, 0.4)',
                    borderColor: isSeries ? 'rgb(59, 130, 246)' : 'rgb(147, 51, 234)',
                    borderWidth: 2,
                    fill: isSeries,
                    tension: 0.4,
                    pointRadius: isSeries ? 3 : 0
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: { beginAtZero: true, grid: { color: 'rgba(128,128,128,0.05)' } },
                    x: { grid: { display: false }, ticks: { display: widget.width_span > 1 } }
                },
                plugins: { legend: { display: false } }
            }
        });
    }

    onMount(loadWidgetData);
    $effect(() => { if (canvas && data.length > 0) renderChart(canvas, data); });
</script>

<div class="h-full w-full" style="grid-column: span {widget.width_span}">
    <Card title={widget.title} subtitle={widget.config.semantic_type}>
        <div class="h-48 w-full relative">
            {#if loading}
                <div class="absolute inset-0 flex justify-center items-center opacity-20">
                    <span class="loading loading-spinner loading-md"></span>
                </div>
            {:else if data.length === 0}
                <div class="absolute inset-0 flex justify-center items-center opacity-10 italic text-[10px] uppercase font-black tracking-widest">
                    No data
                </div>
            {:else}
                <canvas bind:this={canvas}></canvas>
            {/if}
        </div>
    </Card>
</div>
