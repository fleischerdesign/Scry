<script lang="ts">
	import Icon from "@iconify/svelte";

	interface Props {
		icon: string;
		title: string;
		subtitle: string;
		onclick: () => void;
	}

	let { icon, title, subtitle, onclick }: Props = $props();
</script>

<button
	onclick={onclick}
	class="group p-6 bg-base-100 border border-base-300 rounded-2xl hover:border-primary transition-all text-left"
>
	<div class="flex items-center justify-between mb-4">
		<div class="w-12 h-12 rounded-2xl bg-primary/5 text-primary flex items-center justify-center group-hover:scale-110 transition-transform shadow-sm">
			<Icon icon={icon} class="w-6 h-6" />
		</div>
		<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 opacity-0 group-hover:opacity-100 text-primary transition-opacity" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" /></svg>
	</div>
	<h3 class="font-black text-lg tracking-tight mb-1">{title}</h3>
	<p class="text-xs opacity-70 leading-relaxed">{subtitle}</p>
</button>
