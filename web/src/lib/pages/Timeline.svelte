<script lang="ts">
    import TimelineItem from "../components/TimelineItem.svelte";
    import { timeline } from "../state/timeline.svelte";
    let { onRefresh } = $props();
</script>

<div class="space-y-10 animate-in fade-in duration-500 w-full">
    <div class="flex items-center justify-between px-2">
        <h2 class="text-xs font-black uppercase tracking-[0.3em] opacity-30">Temporal_Stream</h2>
        <button class="btn btn-ghost btn-xs font-mono opacity-30 hover:opacity-100 uppercase tracking-widest" onclick={onRefresh}>Sync_</button>
    </div>

    <ul class="timeline timeline-vertical timeline-compact">
        {#each timeline.items as item, i}
            <TimelineItem 
                {item} 
                isFirst={i === 0} 
                isLast={i === timeline.items.length - 1} 
            />
        {/each}
    </ul>

    {#if timeline.items.length === 0}
        <div class="flex flex-col items-center justify-center py-20 opacity-20 border-2 border-dashed border-base-300 rounded-3xl">
            <p class="font-mono text-xs uppercase tracking-widest">No events in current epoch</p>
        </div>
    {/if}
</div>
