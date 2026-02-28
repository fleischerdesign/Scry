<script lang="ts">
	import { auth } from "../auth.svelte";

	let { onAuthSuccess } = $props();

	let username = $state("");
	let password = $state("");
	let isRegistering = $state(false);
	let error = $state("");
	let loading = $state(false);

	async function handleAuth() {
		loading = true;
		error = "";
		const endpoint = isRegistering ? "register" : "login";
		try {
			const res = await fetch(`http://127.0.0.1:3000/api/v1/auth/${endpoint}`, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({ username, password })
			});
			const data = await res.json();
			if (!res.ok) throw new Error(data.error || "Auth failed");
			auth.login(data.api_key, data.user);
			onAuthSuccess();
		} catch (e: any) {
			error = e.message;
		} finally {
			loading = false;
		}
	}
</script>

<div class="hero min-h-screen bg-base-300">
	<div class="hero-content flex-col">
		<div class="text-center mb-8">
			<h1 class="text-5xl font-bold font-mono tracking-tighter italic">SCRY_</h1>
			<p class="py-6 opacity-70 text-xl text-balance tracking-tight">Your Life, Digitally Orchestrated.</p>
		</div>
		<div class="card shrink-0 w-full max-w-sm shadow-2xl bg-base-100 border border-base-300">
			<form class="card-body" onsubmit={(e) => { e.preventDefault(); handleAuth(); }}>
				<h2 class="card-title justify-center mb-4 text-xs uppercase tracking-[0.2em] opacity-50 font-black">{isRegistering ? 'Create Node' : 'Initialize Session'}</h2>
				<div class="form-control">
					<label class="label" for="username"><span class="label-text font-bold text-[10px] uppercase opacity-40">Identifier</span></label>
					<input type="text" id="username" bind:value={username} placeholder="philipp" class="input input-bordered font-mono text-sm" required />
				</div>
				<div class="form-control mt-2">
					<label class="label" for="password"><span class="label-text font-bold text-[10px] uppercase opacity-40">Secret</span></label>
					<input type="password" id="password" bind:value={password} placeholder="••••••••" class="input input-bordered font-mono text-sm" required />
				</div>
				{#if error}
					<div class="alert alert-error mt-4 py-2 text-[10px] font-mono rounded-lg">
						<span>{error}</span>
					</div>
				{/if}
				<div class="form-control mt-6">
					<button class="btn btn-primary" type="submit" disabled={loading}>
						{#if loading}<span class="loading loading-spinner loading-xs"></span>{/if}
						{isRegistering ? 'Register' : 'Login'}
					</button>
				</div>
				<div class="divider text-[9px] opacity-20 font-mono uppercase tracking-widest">Protocol</div>
				<button 
					type="button" 
					class="btn btn-ghost btn-sm font-mono text-[10px] opacity-40 hover:opacity-100"
					onclick={() => isRegistering = !isRegistering}
				>
					{isRegistering ? 'Existing Identity? Login' : 'New Identity? Register'}
				</button>
			</form>
		</div>
	</div>
</div>
