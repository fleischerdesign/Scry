<script lang="ts">
	interface Props {
		label?: string;
	}

	let { label }: Props = $props();
</script>

<div class="flex flex-col items-center justify-center py-20 opacity-60">
	<span class="loading loading-spinner loading-lg text-primary"></span>
	{#if label}
		<p class="text-xs font-bold tracking-wide mt-4">{label}</p>
	{/if}
</div>
