<script lang="ts">
	import Icon from "@iconify/svelte";

	interface Props {
		icon: string;
		title: string;
		description?: string;
		cta?: string;
		oncta?: () => void;
	}

	let { icon, title, description, cta, oncta }: Props = $props();
</script>

<div class="flex flex-col items-center justify-center py-32 opacity-60 border-2 border-dashed border-base-300 rounded-3xl">
	<Icon icon={icon} class="w-12 h-12 mb-4" />
	<h2 class="text-sm font-bold tracking-wide">{title}</h2>
	{#if description}
		<p class="text-xs opacity-70 mt-2 max-w-sm text-center">{description}</p>
	{/if}
	{#if cta && oncta}
		<button class="btn btn-primary btn-sm mt-6 px-8" onclick={oncta}>{cta}</button>
	{/if}
</div>
