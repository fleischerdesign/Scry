<script lang="ts">
 let { title, value, desc, trend = "", color = "primary" } = $props();
</script>

<div class="stats shadow bg-base-100 border border-base-300 w-full">
 <div class="stat">
  <div class="stat-title text-xs font-bold tracking-wide opacity-70">{title}</div>
  <div class="stat-value text-{color} text-2xl tracking-tighter">{value}</div>
  <div class="stat-desc text-xs opacity-70 mt-1 flex justify-between items-center">
   <span>{desc}</span>
   {#if trend}
    <span class="badge badge-ghost badge-xs font-bold">{trend}</span>
   {/if}
  </div>
 </div>
</div>
