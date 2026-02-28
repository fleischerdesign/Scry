<script lang="ts">
    import Card from "../components/Card.svelte";
    import { api } from '../api';
    let { plugins = [], onPoll } = $props();

    let selectedReport = $state<{plugin: string, id: string, name: string} | null>(null);
    let reportData = $state<any>(null);
    let loadingReport = $state(false);

    async function loadReport(plugin: string, report: any) {
        selectedReport = { plugin, id: report.id, name: report.name };
        loadingReport = true;
        try {
            reportData = await api.getPluginReport(plugin, report.id);
        } catch (e) { console.error(e); } finally { loadingReport = false; }
    }
</script>

<div class="space-y-10 animate-in fade-in duration-500 w-full">
    <div class="flex items-center justify-between border-b border-base-300 pb-6">
        <div>
            <h2 class="text-3xl font-black font-mono tracking-tighter italic text-accent">SYSTEM_NODES</h2>
            <p class="text-[10px] uppercase tracking-[0.4em] opacity-30 mt-2 font-black">Kernel Extension Registry</p>
        </div>
    </div>

    <!-- Report Viewer Area -->
    {#if selectedReport}
        <section class="animate-in zoom-in-95 duration-300 mb-12">
            <Card title={selectedReport.name} subtitle={`${selectedReport.plugin.toUpperCase()} / DEEP_REPORT`}>
                {#snippet actions()}
                    <button class="btn btn-ghost btn-xs" onclick={() => selectedReport = null}>Close</button>
                {/snippet}
                {#if loadingReport}
                    <div class="flex justify-center py-10"><span class="loading loading-spinner opacity-20"></span></div>
                {:else if reportData}
                    {@const rows = JSON.parse(reportData.data_json)}
                    <div class="overflow-x-auto mt-4">
                        <table class="table table-zebra table-sm font-mono text-[11px]">
                            <thead>
                                <tr>
                                    {#each reportData.columns as col}<th class="opacity-40">{col}</th>{/each}
                                </tr>
                            </thead>
                            <tbody>
                                {#each rows as row}
                                    <tr>
                                        {#if row.key !== undefined}<td>{row.key}</td>{/if}
                                        {#if row.label !== undefined}<td>{row.label}</td>{/if}
                                        {#if row.count !== undefined}<td>{row.count}</td>{/if}
                                    </tr>
                                {/each}
                            </tbody>
                        </table>
                    </div>
                {/if}
            </Card>
        </section>
    {/if}

    <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
        {#each plugins || [] as plugin}
            <Card title={plugin.name} subtitle={plugin.id}>
                {#snippet actions()}
                    <button class="btn btn-ghost btn-xs text-primary font-mono" onclick={() => onPoll(plugin.id)}>FORCE_POLL</button>
                {/snippet}
                
                <div class="space-y-4">
                    <p class="text-xs opacity-70 leading-relaxed italic">"{plugin.description}"</p>
                    
                    <div class="flex flex-wrap gap-2">
                        {#each plugin.capabilities as cap}
                            <div class="badge badge-outline border-base-300 text-[9px] font-mono uppercase opacity-40">{cap}</div>
                        {/each}
                    </div>

                    {#if plugin.reports.length > 0}
                        <div class="mt-6 space-y-2">
                            <span class="text-[9px] uppercase font-black opacity-30 tracking-widest">Available Deep Reports</span>
                            <div class="flex flex-col gap-1">
                                {#each plugin.reports as report}
                                    <button 
                                        class="btn btn-ghost btn-xs justify-start font-bold text-accent h-auto py-1"
                                        onclick={() => loadReport(plugin.id, report)}
                                    >
                                        → {report.name}
                                    </button>
                                {/each}
                            </div>
                        </div>
                    {/if}
                </div>
            </Card>
        {/each}
    </div>
</div>
