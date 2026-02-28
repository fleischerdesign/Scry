<script lang="ts">
    let { title, value, desc, trend = "", color = "primary" } = $props();
</script>

<div class="stats shadow bg-base-100 border border-base-300 w-full">
    <div class="stat">
        <div class="stat-title text-[10px] uppercase font-black tracking-widest opacity-40">{title}</div>
        <div class="stat-value text-{color} text-2xl font-mono italic tracking-tighter">{value}</div>
        <div class="stat-desc font-mono text-[9px] opacity-50 mt-1 flex justify-between items-center">
            <span>{desc}</span>
            {#if trend}
                <span class="badge badge-ghost badge-xs font-bold">{trend}</span>
            {/if}
        </div>
    </div>
</div>
