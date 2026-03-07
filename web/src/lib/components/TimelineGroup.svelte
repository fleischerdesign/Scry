<script lang="ts">
    import TimelineItem from "./TimelineItem.svelte";
    import type { Event } from "../types/Event";

    interface Props {
        date: string;
        events: Event[];
    }

    let { date, events }: Props = $props();

    function formatHeaderDate(dateStr: string) {
        const d = new Date(dateStr);
        const today = new Date();
        const yesterday = new Date();
        yesterday.setDate(yesterday.getDate() - 1);

        if (d.toDateString() === today.toDateString()) return "Today";
        if (d.toDateString() === yesterday.toDateString()) return "Yesterday";

        return d.toLocaleDateString(undefined, { 
            weekday: 'long', 
            year: 'numeric', 
            month: 'long', 
            day: 'numeric' 
        });
    }
</script>

<section id="group-{date}" class="relative">
    <!-- Sticky Date Header -->
    <div class="sticky top-16 z-10 py-4 bg-base-200/80 backdrop-blur-md -mx-4 px-4 flex items-center justify-between">
        <h3 class="text-[10px] font-black uppercase tracking-[0.3em] text-base-content/40 flex items-center gap-3">
            {formatHeaderDate(date)}
        </h3>
        <div class="badge badge-ghost badge-xs font-mono opacity-30 tracking-tighter uppercase italic border-none">
            {events.length} Events
        </div>
    </div>

    <ul class="timeline timeline-vertical timeline-compact">
        {#each events as item, i}
            <TimelineItem
                {item}
                isFirst={i === 0}
                isLast={i === events.length - 1}
            />
        {/each}
    </ul>
</section>
