<script lang="ts">
	import { onMount } from "svelte";
	import { api } from "../../api";
	import Card from "../../components/Card.svelte";
	import PageLoading from "../../components/PageLoading.svelte";
	import { router } from "../../router.svelte";

	let profile = $state<Record<string, any>>({});
	let loading = $state(true);
	let saving = $state(false);
	let successMessage = $state("");

	$effect(() => {
		router.title = "General";
	});

	async function loadData() {
		loading = true;
		try {
			profile = await api.getProfile();
		} catch (e) {
			console.error("Failed to load profile", e);
		} finally {
			loading = false;
		}
	}

	async function saveProfile() {
		saving = true;
		successMessage = "";
		try {
			await api.updateProfile(profile);
			successMessage = "Profile updated successfully";
			setTimeout(() => (successMessage = ""), 3000);
		} catch (e) {
			console.error("Failed to save profile", e);
		} finally {
			saving = false;
		}
	}

	onMount(loadData);
</script>

<div class="space-y-6 animate-in slide-in-from-right-4 duration-300">
	{#if successMessage}
		<div
			class="badge badge-success text-xs py-3 px-4 animate-bounce fixed top-24 right-10 z-50"
		>
			{successMessage}
		</div>
	{/if}

	{#if loading}
		<PageLoading />
	{:else}
		<Card title="User Profile" subtitle="Profile">
			<div class="grid grid-cols-1 md:grid-cols-2 gap-6 py-2">
				<div class="form-control w-full">
					<label class="label" for="profile-name">
						<span class="label-text text-xs font-bold opacity-70"
							>Display Name</span
						>
					</label>
					<input
						type="text"
						id="profile-name"
						bind:value={profile["name"]}
						placeholder="Your Name"
						class="input input-bordered input-sm font-mono"
					/>
				</div>
				<div class="form-control w-full">
					<label class="label" for="profile-avatar">
						<span class="label-text text-xs font-bold opacity-70"
							>Avatar URL</span
						>
					</label>
					<input
						type="text"
						id="profile-avatar"
						bind:value={profile["avatar"]}
						placeholder="https://..."
						class="input input-bordered input-sm font-mono"
					/>
				</div>
				<div class="form-control w-full">
					<label class="label" for="profile-city">
						<span class="label-text text-xs font-bold opacity-70"
							>Home City (Global)</span
						>
					</label>
					<input
						type="text"
						id="profile-city"
						bind:value={profile["city"]}
						placeholder="Berlin, London..."
						class="input input-bordered input-sm font-mono"
					/>
				</div>
			</div>
			{#snippet actions()}
				<button
					class="btn btn-primary btn-sm"
					onclick={saveProfile}
					disabled={saving}
				>
					{saving ? "Saving..." : "Save Profile"}
				</button>
			{/snippet}
		</Card>
	{/if}
</div>
