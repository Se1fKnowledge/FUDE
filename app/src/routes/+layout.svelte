<script lang="ts">
	import '#lib/styles.css';
	import { onMount, type Snippet } from 'svelte';
	import { page } from '$app/state';
	import Sidebar from '#lib/components/Sidebar.svelte';
	import Splash from '#lib/components/Splash.svelte';
	import Welcome from '#lib/components/Welcome.svelte';
	import { refreshSession, session } from '#lib/session.svelte.ts';
	import { api } from '#lib/api.ts';

	let { children }: { children: Snippet } = $props();

	let splash = $state(true);

	onMount(() => {
		// The splash plays once per launch, not on every reload while developing.
		if (sessionStorage.getItem('fude-splash')) splash = false;
		sessionStorage.setItem('fude-splash', '1');
		refreshSession();
	});

	// The lesson path takes the whole window, like the mockup screens 06–08.
	const focus = $derived(/^\/(lesson|practice|result)\//.test(page.url.pathname));

	// Leaving the lesson path closes the editor it opened.
	$effect(() => {
		if (!focus && session.boot) api.editorClose().catch(() => {});
	});
</script>

{#if splash}
	<Splash ondone={() => (splash = false)} />
{/if}

{#if session.boot?.firstRun && !splash}
	<Welcome ondone={refreshSession} />
{/if}

{#if session.error && !session.boot}
	<main class="fatal">
		<h1 class="h1">FUDE не запустился</h1>
		<p class="muted">{session.error}</p>
	</main>
{:else if focus}
	{@render children()}
{:else}
	<div class="shell">
		<Sidebar />
		<main>{@render children()}</main>
	</div>
{/if}

<style>
	.shell {
		display: grid;
		grid-template-columns: var(--sidebar) 1fr;
		min-height: 100vh;
	}

	main {
		min-width: 0;
	}

	.fatal {
		display: grid;
		gap: 12px;
		place-content: center;
		min-height: 100vh;
		padding: 40px;
	}
</style>
