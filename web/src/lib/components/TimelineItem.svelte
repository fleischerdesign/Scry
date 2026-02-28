<script lang="ts">
    let { item, isFirst = false, isLast = false } = $props();

    function getEventLabel(item: any) {
        const e = item.event;
        if (!e) return 'Unknown Data';
        if (item.category === "music.scrobble") return `${e.artist || 'Unknown'} - ${e.track || 'Unknown'}`;
        if (item.category === "weather.current") return `${e.temperature}°C (${e.time || 'now'})`;
        return e.track || e.temperature || e.message || JSON.stringify(e).slice(0, 30);
    }

    const time = $derived(new Date(item.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }));
</script>

<li>
    {#if !isFirst}<hr class="bg-base-300" />{/if}
    <div class="timeline-start font-mono text-[10px] opacity-40">{time}</div>
    <div class="timeline-middle">
        <div class="w-3 h-3 rounded-full bg-primary/40 ring-4 ring-primary/10"></div>
    </div>
    <div class="timeline-end timeline-box border-none bg-base-100 shadow-none py-4 px-2 hover:bg-base-300/30 transition-colors rounded-xl w-full">
        <div class="flex flex-col gap-1">
            <div class="flex justify-between items-center">
                <span class="text-xs font-black tracking-tight">{getEventLabel(item)}</span>
                <span class="badge badge-ghost badge-xs font-mono opacity-30 italic lowercase">{item.category?.split('.').pop()}</span>
            </div>
            <div class="flex flex-wrap gap-1 mt-1">
                {#each Object.entries(item.context || {}) as [ctxType, ctxVal]}
                    <div class="badge badge-outline border-base-300 text-[9px] font-mono h-4 gap-1 opacity-60">
                        <span class="opacity-40 uppercase font-bold text-[7px]">{ctxType.split('.').pop()}:</span>
                        <span>{(ctxVal as any).temperature || (ctxVal as any).track || '...'}</span>
                    </div>
                {/each}
            </div>
        </div>
    </div>
    {#if !isLast}<hr class="bg-base-300" />{/if}
</li>
