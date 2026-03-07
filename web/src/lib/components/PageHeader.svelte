<script lang="ts">
    import type { Snippet } from "svelte";
    import Icon from "@iconify/svelte";

    interface Props {
        title: string;
        subtitle?: string;
        image?: Snippet;
        onBack?: () => void;
        children?: Snippet;
        actions?: Snippet;
    }

    let { title, subtitle, image, onBack, children, actions }: Props = $props();
</script>

<header class="flex flex-col lg:flex-row lg:items-center justify-between gap-6 border-b border-base-300 pb-8 mb-8">
    <div class="flex items-center gap-6">
        {#if onBack}
            <button 
                class="btn btn-ghost btn-sm btn-square rounded-xl border border-base-300 hover:bg-base-300 transition-all shrink-0" 
                onclick={onBack}
                aria-label="Go back"
            >
                <Icon icon="lucide:arrow-left" class="w-4 h-4" />
            </button>
        {/if}

        {#if image}
            <div class="shrink-0">
                {@render image()}
            </div>
        {/if}

        <div class="space-y-1 flex-1">
            <div class="flex items-center gap-3">
                <h1 class="text-3xl font-bold tracking-tight text-base-content leading-tight">
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
    </div>

    {#if actions}
        <div class="flex items-center gap-3 shrink-0">
            {@render actions()}
        </div>
    {/if}
</header>
