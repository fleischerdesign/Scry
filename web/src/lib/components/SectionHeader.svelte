<script lang="ts">
	interface Props {
		label: string;
	}

	let { label }: Props = $props();
</script>

<h3 class="text-xs font-black tracking-wide opacity-60 px-2">{label}</h3>
