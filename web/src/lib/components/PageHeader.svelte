<script lang="ts">
    import type { Snippet } from "svelte";

    interface Props {
        title: string;
        subtitle?: string;
        children?: Snippet;
        actions?: Snippet;
    }

    let { title, subtitle, children, actions }: Props = $props();
</script>

<header class="flex flex-col lg:flex-row lg:items-end justify-between gap-6 border-b border-base-300 pb-8 mb-8">
    <div class="space-y-1">
        <div class="flex items-center gap-3">
            <h1 class="text-3xl font-bold tracking-tight text-base-content">
                {title}
            </h1>
        </div>
        
        {#if subtitle}
            <p class="text-sm text-base-content/50 max-w-2xl leading-relaxed">
                {subtitle}
            </p>
        {/if}

        {#if children}
            <div class="pt-2">
                {@render children()}
            </div>
        {/if}
    </div>

    {#if actions}
        <div class="flex items-center gap-3 shrink-0">
            {@render actions()}
        </div>
    {/if}
</header>
