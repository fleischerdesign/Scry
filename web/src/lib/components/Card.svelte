<script lang="ts">
 import type { Snippet } from 'svelte';
 interface Props {
  title?: string;
  subtitle?: string;
  children: Snippet;
  actions?: Snippet;
  header?: Snippet;
 }
 let { title = "", subtitle = "", children, actions, header }: Props = $props();
</script>

<div class="card card-bordered bg-base-100 shadow-sm hover:shadow-md transition-shadow overflow-hidden">
 {#if header}
  <div class="p-6 bg-base-200/50 border-b border-base-300 flex items-center gap-3">
   {@render header()}
  </div>
 {/if}
 <div class="card-body">
  {#if title || subtitle}
   <div class="flex justify-between items-start">
    <div>
     {#if subtitle}
      <div class="text-xs font-bold opacity-70 tracking-wide">{subtitle}</div>
     {/if}
     {#if title}
      <h2 class="card-title text-base">{title}</h2>
     {/if}
    </div>
    {#if actions}
     <div class="card-actions justify-end">
      {@render actions()}
     </div>
    {/if}
   </div>
  {/if}
  
  <div class="mt-2">
   {@render children()}
  </div>
 </div>
</div>
