<script lang="ts">
    import { Chart, registerables } from 'chart.js';
    import CorrelationCard from "../components/CorrelationCard.svelte";
    
    Chart.register(...registerables);

    let { statsData } = $props();
    let canvas = $state<HTMLCanvasElement>();
    let chart: Chart | undefined;

    $effect(() => {
        if (canvas && statsData) {
            if (chart) chart.destroy();
            const firstKey = Object.keys(statsData.correlations)[0];
            if (!firstKey) return;
            const joins = statsData.correlations[firstKey];
            chart = new Chart(canvas, {
                type: 'bar',
                data: {
                    labels: Object.keys(joins).map(v => { try { return `${JSON.parse(v).temperature}°C`; } catch { return v; } }),
                    datasets: [{ data: Object.values(joins), backgroundColor: 'rgba(255, 117, 184, 0.4)', borderColor: 'rgb(255, 117, 184)', borderWidth: 2 }]
                },
                options: { 
                    responsive: true, 
                    maintainAspectRatio: false, 
                    scales: { y: { beginAtZero: true, grid: { color: 'rgba(128,128,128,0.05)' } } }, 
                    plugins: { legend: { display: false } } 
                }
            });
        }
        return () => { if (chart) chart.destroy(); };
    });
</script>

<div class="space-y-12 animate-in slide-in-from-bottom-4 duration-700 w-full">
    <div class="flex items-center justify-between border-b border-base-300 pb-6">
        <div>
            <h2 class="text-3xl font-black font-mono tracking-tighter italic text-primary">INSIGHTS_</h2>
            <p class="text-[10px] uppercase tracking-[0.4em] opacity-30 mt-2 font-black">Pattern Recognition & Machine Learning</p>
        </div>
        <div class="stats bg-base-100 border border-base-300 shadow-sm overflow-hidden">
            <div class="stat py-2 px-6">
                <div class="stat-title text-[9px] uppercase font-bold opacity-40">Sample Size</div>
                <div class="stat-value text-sm font-mono">{statsData?.sample_size || 0}</div>
            </div>
        </div>
    </div>

    <!-- Global Frequency Correlation -->
    <section class="space-y-6">
        <div class="flex items-center gap-4 px-2">
            <div class="w-8 h-px bg-base-300"></div>
            <h3 class="text-xs font-black uppercase tracking-widest opacity-40">Cross-Semantic distribution</h3>
        </div>
        <div class="card bg-base-100 border border-base-300 shadow-sm overflow-hidden">
            <div class="card-body p-0">
                <div class="p-6 border-b border-base-300 bg-base-200/30 flex justify-between items-center">
                    <h3 class="text-xs font-black uppercase tracking-widest opacity-50">Event frequency vs Context</h3>
                    <div class="badge badge-primary badge-xs py-2 px-3 font-mono">CORE_MATRIX</div>
                </div>
                <div class="h-96 w-full p-8">
                    <canvas bind:this={canvas}></canvas>
                </div>
            </div>
        </div>
    </section>

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
        {#each Object.entries(statsData?.correlations || {}) as [baseVal, joins]}
            <CorrelationCard {baseVal} {joins} sampleSize={statsData.sample_size} />
        {/each}
    </div>
</div>
