<script lang="ts">
	import { router } from "../router.svelte";
	import { plugins } from "../state/plugins.svelte";
	import { dashboards } from "../state/dashboards.svelte";

	$effect(() => {
		router.title = "Settings";
	});
</script>

<div
	class="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500 w-full max-w-4xl"
>
	<!-- Home Assistant Style Vertical List -->
	<div
		class="flex flex-col bg-base-200 rounded-3xl overflow-hidden border border-base-300 divide-y divide-base-300/50"
	>
		<button
			onclick={() => router.navigate("/settings/general")}
			class="flex items-center gap-4 p-5 hover:bg-base-300 transition-all group text-left w-full"
		>
			<div
				class="w-10 h-10 rounded-2xl bg-primary/10 flex items-center justify-center text-primary group-hover:scale-105 transition-transform"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-6 w-6"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
					/>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
					/>
				</svg>
			</div>
			<div class="flex-1">
				<h3 class="font-bold text-sm uppercase tracking-tight italic">
					General
				</h3>
				<p class="text-[10px] opacity-40 uppercase font-mono tracking-widest">
					Identity, Appearance & API Keys
				</p>
			</div>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				class="h-4 w-4 opacity-20"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
				><path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M9 5l7 7-7 7"
				/></svg
			>
		</button>

		<button
			onclick={() => router.navigate("/settings/plugins")}
			class="flex items-center gap-4 p-5 hover:bg-base-300 transition-all group text-left w-full"
		>
			<div
				class="w-10 h-10 rounded-2xl bg-secondary/10 flex items-center justify-center text-secondary group-hover:scale-105 transition-transform"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-6 w-6"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z"
					/>
				</svg>
			</div>
			<div class="flex-1">
				<h3 class="font-bold text-sm uppercase tracking-tight italic">
					Extensions
				</h3>
				<p class="text-[10px] opacity-40 uppercase font-mono tracking-widest">
					Manage WASM Nodes, Sources & Enrichers
				</p>
			</div>
			<div class="flex items-center gap-3">
				<div class="badge badge-secondary badge-outline badge-sm opacity-40 font-mono text-[9px]">{plugins.items.length}</div>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-4 w-4 opacity-20"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
					><path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M9 5l7 7-7 7"
					/></svg
				>
			</div>
		</button>

		<button
			onclick={() => router.navigate("/settings/dashboards")}
			class="flex items-center gap-4 p-5 hover:bg-base-300 transition-all group text-left w-full"
		>
			<div
				class="w-10 h-10 rounded-2xl bg-warning/10 flex items-center justify-center text-warning group-hover:scale-105 transition-transform"
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-6 w-6"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z"
					/>
				</svg>
			</div>
			<div class="flex-1">
				<h3 class="font-bold text-sm uppercase tracking-tight italic">
					Dashboards
				</h3>
				<p class="text-[10px] opacity-40 uppercase font-mono tracking-widest">
					Layouts, Widgets & Visualization
				</p>
			</div>
			<div class="flex items-center gap-3">
				<div class="badge badge-warning badge-outline badge-sm opacity-40 font-mono text-[9px]">{dashboards.items.length}</div>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-4 w-4 opacity-20"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
					><path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M9 5l7 7-7 7"
					/></svg
				>
			</div>
		</button>
	</div>
</div>
