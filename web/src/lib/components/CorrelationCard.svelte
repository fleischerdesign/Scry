<script lang="ts">
    let { baseVal, joins, sampleSize } = $props();
    
    function parseValue(val: string) {
        try {
            return JSON.parse(val);
        } catch {
            return val;
        }
    }

    const baseParsed = $derived(parseValue(baseVal));
</script>

<div class="card bg-base-100 border border-base-300 shadow-sm hover:shadow-md transition-shadow">
    <div class="card-body p-6">
        <div class="flex items-center gap-3 mb-6">
            <div class="w-1 h-8 bg-secondary rounded-full"></div>
            <div>
                <h3 class="text-xs uppercase font-bold opacity-40 tracking-tighter">Context Cluster</h3>
                <p class="text-sm font-black font-mono truncate max-w-xs italic text-secondary">
                    {baseParsed.artist || baseVal}
                </p>
            </div>
        </div>
        
        <div class="space-y-5">
            {#each Object.entries(joins as any) as [joinVal, count]}
                {@const joinParsed = parseValue(joinVal)}
                <div class="space-y-2">
                    <div class="flex justify-between items-end">
                        <span class="text-xs font-medium font-mono text-base-content/70">
                            {joinParsed.temperature || joinVal} °C
                        </span>
                        <span class="text-[10px] font-bold opacity-30 italic">{count} occurrences</span>
                    </div>
                    <progress 
                        class="progress progress-secondary w-full h-1.5 opacity-30 hover:opacity-100 transition-opacity" 
                        value={count as number} 
                        max={sampleSize}
                    ></progress>
                </div>
            {/each}
        </div>
    </div>
</div>
