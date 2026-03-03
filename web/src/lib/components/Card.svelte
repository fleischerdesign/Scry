<script lang="ts">
    import type { Snippet } from 'svelte';
    interface Props {
        title?: string;
        subtitle?: string;
        children: Snippet;
        actions?: Snippet;
    }
    let { title = "", subtitle = "", children, actions }: Props = $props();
</script>

<div class="card card-bordered bg-base-100 shadow-sm hover:shadow-md transition-shadow">
    <div class="card-body">
        {#if title || subtitle}
            <div class="flex justify-between items-start">
                <div>
                    {#if subtitle}
                        <div class="text-[10px] uppercase font-bold opacity-40 tracking-widest">{subtitle}</div>
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
