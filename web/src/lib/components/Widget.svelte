<script lang="ts">
  import { onMount } from 'svelte';
  import { Chart, registerables } from 'chart.js';
  import { api } from '../api';
  import { semanticService } from '../services/semantic.svelte';
  import Card from './Card.svelte';
  import { getThemeCSS } from '../theme';
 
  Chart.register(...registerables);
 
  let { widget } = $props();
  const parsedConfig = $derived(typeof widget.config === 'string' ? JSON.parse(widget.config) : widget.config);
  
  let canvas = $state<HTMLCanvasElement>();
  let chart: Chart | undefined;
  let loading = $state(true);
  let data = $state<any[]>([]);
   let latestValue = $state<any>(null);
   let latestConfidence = $state<number | null>(null);
   let displayValue = $state<string>('---');
   let pluginsStatus = $state<any[]>([]);
    
    async function loadWidgetData() {
     loading = true;
     try {
      const { semantic_type, category, path, days = 7 } = parsedConfig;
   
      if (widget.type === 'Status') {
       pluginsStatus = await api.getPlugins();
      } else if (semantic_type === 'system.entities') {
       // Simulating growth (Future: Core API)
       data = Array.from({length: days}, (_, i) => ({
        label: new Date(Date.now() - (days - 1 - i) * 86400000).toLocaleDateString(),
        count: Math.floor(Math.random() * 50) + 10
       }));
      } else if (widget.type === 'Trend' || widget.type === 'semantic_series') {
       data = await api.getSemanticSeries(semantic_type, days);
      } else if (widget.type === 'TopList' || widget.type === 'semantic_top') {
       data = await api.getSemanticTop(semantic_type, 10, days);
      } else if (widget.type === 'Metric' || widget.type === 'stat') {
       const queryPath = semantic_type || category;
       if (!queryPath) return;
       
       const latest = await api.getData(queryPath, 1);
       if (latest && latest.length > 0) {
        const event = latest[0];
        latestValue = event.display_value;
        latestConfidence = event.confidence;
        
        // Semantic Formatting
        displayValue = semanticService.formatValue(latestValue, {
         semantic_type: semantic_type,
         unit: parsedConfig.unit,
         privacy: parsedConfig.privacy
        });
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

  const primary = getThemeCSS("--color-primary", "#3b82f6");
  const gridColor = getThemeCSS("--color-base-300", "#d1d5db");

  const labels = plotData.map(d => d.display_title || d.label || d.key);
  const values = plotData.map(d => d.value !== undefined ? d.value : d.count);

  chart = new Chart(node, {
   type: widget.type === 'Trend' || widget.type === 'semantic_series' ? 'line' : 'bar',
   data: {
    labels,
    datasets: [{
     data: values,
     backgroundColor: primary + '20',
     borderColor: primary,
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
     y: { beginAtZero: true, grid: { color: gridColor + '20' }, ticks: { font: { size: 10 } } },
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
    <div class="absolute inset-0 flex justify-center items-center opacity-60">
     <span class="loading loading-spinner loading-md text-primary"></span>
    </div>
   {:else if widget.type === 'Metric' || widget.type === 'stat'}
    <div class="flex flex-col items-center">
     <span class="text-5xl font-bold tracking-tighter text-primary {latestConfidence !== null && latestConfidence < 0.9 ? 'opacity-70' : ''}">
      {displayValue}
     </span>
    </div>
   {:else if widget.type === 'Status'}
    <div class="space-y-2 overflow-y-auto max-h-full px-2">
     {#each pluginsStatus as p}
      <div class="flex justify-between items-center bg-base-200 p-2 rounded-xl border border-base-300/50">
       <div class="flex flex-col overflow-hidden">
        <span class="text-xs font-bold truncate">{p.name}</span>
        <span class="text-xs font-mono opacity-70">{p.id}</span>
       </div>
       <div class="badge badge-success badge-xs"></div>
      </div>
     {/each}
    </div>
   {:else if (widget.type === 'TopList' || widget.type === 'semantic_top') && data.length > 0}
    <div class="space-y-2 overflow-y-auto max-h-full px-2">
     {#each data.slice(0, 5) as item}
      <div class="flex justify-between items-center text-[11px] gap-2">
       <div class="flex items-center gap-2 flex-1 min-w-0">
        {#if item.display_image}
         <div class="w-5 h-5 rounded-md overflow-hidden bg-base-300 shrink-0">
          <img src={item.display_image} alt="" class="object-cover w-full h-full" />
         </div>
        {:else}
         <div class="w-5 h-5 rounded-md bg-base-300 flex items-center justify-center text-xs font-bold opacity-70 shrink-0">
          {(item.display_title || item.key).charAt(0).toUpperCase()}
         </div>
        {/if}
        <span class="font-bold truncate">{item.display_title || item.key}</span>
       </div>
       <div class="flex items-center gap-2">
        <div class="w-12 h-1 bg-base-300 rounded-full overflow-hidden">
         <div class="bg-secondary h-full" style="width: {(item.count / data[0].count) * 100}%"></div>
        </div>
        <span class="opacity-70 font-mono">{item.count}</span>
       </div>
      </div>
     {/each}
    </div>
   {:else if data.length > 0}
    <canvas bind:this={canvas}></canvas>
   {:else}
    <div class="absolute inset-0 flex justify-center items-center opacity-40 text-xs font-bold tracking-wide">
     No data available
    </div>
   {/if}
  </div>
 </Card>
</div>
