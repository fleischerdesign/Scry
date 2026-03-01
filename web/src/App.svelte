<script lang="ts">
	import "./app.css";
	import { auth } from "./lib/auth.svelte";
	import { onMount } from "svelte";
	import { router } from "./lib/router.svelte";
	import { ui } from "./lib/ui.svelte";
	import { fly } from "svelte/transition";
	
	// Global States
	import { timeline } from "./lib/state/timeline.svelte";
	import { plugins } from "./lib/state/plugins.svelte";
	import { dashboards } from "./lib/state/dashboards.svelte";

	// Pages
	import Overview from "./lib/pages/Overview.svelte";
	import Timeline from "./lib/pages/Timeline.svelte";
	import Explorer from "./lib/pages/Explorer.svelte";
	import Dashboard from "./lib/pages/Dashboard.svelte";
	import Analytics from "./lib/pages/Analytics.svelte";
	import Settings from "./lib/pages/Settings.svelte";
	import GeneralSettings from "./lib/pages/settings/GeneralSettings.svelte";
	import PluginSettings from "./lib/pages/settings/PluginSettings.svelte";
	import EnricherSettings from "./lib/pages/settings/EnricherSettings.svelte";
	import DashboardSettings from "./lib/pages/settings/DashboardSettings.svelte";
	import EventDetail from "./lib/pages/EventDetail.svelte";
	import EntityDetail from "./lib/pages/EntityDetail.svelte";
	import Auth from "./lib/components/Auth.svelte";
	import CommandPalette from "./lib/components/CommandPalette.svelte";

	let isPaletteOpen = $state(false);
	let currentTheme = $state(localStorage.getItem("scry_theme") || "dark");

	$effect(() => {
		document.documentElement.setAttribute("data-theme", currentTheme);
		localStorage.setItem("scry_theme", currentTheme);
	});

	async function loadAll() {
		if (!auth.isAuthenticated) return;
		await Promise.all([
			timeline.load(),
			plugins.load(),
			dashboards.load()
		]);
	}

	onMount(() => { if (auth.isAuthenticated) loadAll(); });
</script>

<CommandPalette bind:isOpen={isPaletteOpen} onAction={(type, payload) => {
	if (type === 'nav') router.navigate(payload);
}} />

{#if !auth.isAuthenticated}
	<Auth onAuthSuccess={loadAll} />
{:else}
	<div class="drawer lg:drawer-open">
		<input id="main-drawer" type="checkbox" class="drawer-toggle" />
		<div class="drawer-content flex flex-col bg-base-200 min-h-screen">
			<div class="navbar bg-base-100 border-b border-base-300 px-4 sticky top-0 z-20">
				<div class="navbar-start">
					<label for="main-drawer" class="btn btn-square btn-ghost lg:hidden"><svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="inline-block w-5 h-5 stroke-current"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path></svg></label>
					<div class="hidden lg:flex px-4"><h1 class="text-[10px] font-black uppercase tracking-[0.2em] opacity-40">{router.path.split('/').pop() || 'overview'}</h1></div>
				</div>
				<div class="navbar-center hidden sm:flex w-full max-w-md">
					<button class="btn btn-sm btn-ghost bg-base-200 w-full justify-between font-normal text-base-content/40 hover:bg-base-300 border-base-300" onclick={() => isPaletteOpen = true}>
						<span class="flex items-center gap-2"><svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" /></svg>Search anything...</span>
						<kbd class="kbd kbd-xs bg-base-100 opacity-50">CTRL K</kbd>
					</button>
				</div>
				<div class="navbar-end">
					<div class="dropdown dropdown-end">
						<div tabindex="0" role="button" class="btn btn-ghost btn-sm font-mono uppercase tracking-tighter opacity-60">Theme</div>
						<ul tabindex="0" class="dropdown-content z-[1] p-2 shadow-2xl bg-base-300 rounded-box w-52 mt-4 border border-base-300">
							{#each ['default', 'light', 'dark', 'retro', 'cyberpunk', 'synthwave', 'luxury', 'business'] as theme}
								<li><input type="radio" name="theme-dropdown" class="theme-controller btn btn-sm btn-block btn-ghost justify-start font-mono text-[10px] uppercase" aria-label={theme} value={theme} checked={currentTheme === theme} onclick={() => currentTheme = theme}/></li>
							{/each}
						</ul>
					</div>
				</div>
			</div>

			<main class="p-6 lg:p-10 w-full flex-1">
				{#if router.path === '/overview'}
					<Overview onRefresh={loadAll} />
				{:else if router.path === '/timeline'}
					<Timeline onRefresh={loadAll} />
				{:else if router.path === '/explorer'}
					<Explorer onRefresh={loadAll} />
				{:else if router.path.startsWith('/dashboard/')}
					<Dashboard onRefresh={loadAll} />
				{:else if router.path === '/analytics'}
					<Analytics />
				{:else if router.path.startsWith('/settings')}
					<div class="space-y-6">
						<div class="flex flex-col gap-4 border-b border-base-300 pb-6">
							<div class="text-xs breadcrumbs font-mono opacity-40 uppercase tracking-widest">
								<ul>
									<li><button onclick={() => router.navigate('/settings')}>Settings</button></li>
									{#if router.path !== '/settings'}
										<li>{router.path.split('/').pop()}</li>
									{/if}
								</ul>
							</div>
							<h2 class="text-3xl font-black font-mono tracking-tighter italic text-secondary uppercase">
								{router.path === '/settings' ? 'CONTROL_CENTER' : router.path.split('/').pop()?.toUpperCase()}_
							</h2>
						</div>

						{#if router.path === '/settings'}
							<Settings />
						{:else if router.path === '/settings/general'}
							<GeneralSettings />
						{:else if router.path === '/settings/plugins'}
							<PluginSettings />
						{:else if router.path === '/settings/enrichers'}
							<EnricherSettings />
						{:else if router.path === '/settings/dashboards'}
							<DashboardSettings onRefresh={loadAll} />
						{/if}
					</div>
				{:else if router.match('/event/:id')}
					<EventDetail />
				{:else if router.match('/entity/:ns/:type/:id')}
					<EntityDetail />
				{/if}
			</main>
		</div>

		<div class="drawer-side z-30">
			<label for="main-drawer" aria-label="close sidebar" class="drawer-overlay"></label>
			<aside class="menu p-4 w-80 min-h-full bg-base-100 border-r border-base-300 text-base-content flex flex-col">
				<div class="px-4 py-8 mb-4 flex items-center gap-4 border-b border-base-200">
					<div class="w-10 h-10 bg-primary rounded-xl flex items-center justify-center text-primary-content font-black text-2xl italic tracking-tighter shadow-lg shadow-primary/20">S</div>
					<div><h2 class="text-2xl font-black font-mono tracking-tighter italic text-base-content">SCRY_</h2><p class="text-[9px] uppercase tracking-[0.3em] opacity-30 font-bold">Life OS Platform</p></div>
				</div>

				<ul class="space-y-1 flex-1 overflow-y-auto">
					<li class="menu-title opacity-40 text-[10px] uppercase tracking-widest font-black pt-4">Navigation</li>
					<li><button class:active={router.path === '/overview'} onclick={() => router.navigate('/overview')} class="gap-4 font-bold tracking-tight">Overview</button></li>
					<li><button class:active={router.path === '/timeline'} onclick={() => router.navigate('/timeline')} class="gap-4 font-bold tracking-tight">Timeline</button></li>
					<li><button class:active={router.path === '/explorer'} onclick={() => router.navigate('/explorer')} class="gap-4 font-bold tracking-tight">Explorer</button></li>
					<li><button class:active={router.path === '/analytics'} onclick={() => router.navigate('/analytics')} class="gap-4 font-bold tracking-tight">Insights</button></li>
					<li><button class:active={router.path === '/settings'} onclick={() => router.navigate('/settings')} class="gap-4 font-bold tracking-tight">Settings</button></li>
					
					<li class="menu-title opacity-40 text-[10px] uppercase tracking-widest font-black pt-8">Dashboards</li>
					{#each dashboards.items as dash}
						<li><button class:active={router.path === `/dashboard/${dash.slug}`} onclick={() => router.navigate(`/dashboard/${dash.slug}`)} class="gap-4 font-bold tracking-tight text-secondary">
							<div class="w-1 h-1 rounded-full bg-secondary opacity-40"></div>
							{dash.name}
						</button></li>
					{/each}
				</ul>

				<div class="mt-auto pt-4">
					<div class="bg-base-200 rounded-2xl p-4 flex items-center justify-between">
						<div class="flex flex-col"><span class="text-xs font-bold">@{auth.user?.username}</span><span class="text-[9px] opacity-40 uppercase tracking-widest">Authorized Node</span></div>
						<button class="btn btn-ghost btn-circle btn-sm text-error/60" onclick={() => auth.logout()}><svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" /></svg></button>
					</div>
				</div>
			</aside>
		</div>
	</div>
{/if}

<div class="toast toast-end toast-bottom z-[100]">
	{#each ui.toasts as toast (toast.id)}
		<div 
			in:fly={{ y: 20, duration: 300 }} 
			out:fly={{ x: 100, duration: 300 }}
			class="alert"
			class:alert-info={toast.type === 'info'}
			class:alert-success={toast.type === 'success'}
			class:alert-warning={toast.type === 'warning'}
			class:alert-error={toast.type === 'error'}
		>
			<span>{toast.message}</span>
		</div>
	{/each}
</div>

<style>
	:global(.btn) { text-transform: none; }
</style>
