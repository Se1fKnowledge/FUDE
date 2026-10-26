<script lang="ts">
	import { api, errorText } from '#lib/api.ts';
	import Enso from './Enso.svelte';
	import Hanko from './Hanko.svelte';

	// The first launch: how to address the student and how many tasks a day to aim for.
	let { ondone }: { ondone: () => void } = $props();

	let name = $state('');
	let goal = $state(2);
	let error = $state('');
	let busy = $state(false);

	async function save(e: SubmitEvent) {
		e.preventDefault();
		busy = true;
		try {
			await api.profileSave(name, goal);
			ondone();
		} catch (err) {
			error = errorText(err);
		} finally {
			busy = false;
		}
	}
</script>

<div class="welcome reveal">
	<div class="enso"><Enso size={520} width={34} color="rgb(22 22 26 / 7%)" seed={9} /></div>
	<form class="card" onsubmit={save}>
		<Hanko kanji="筆" size={52} />
		<h1 class="display-l">Добро пожаловать в FUDE</h1>
		<p class="muted">
			Здесь вы учитесь рисовать прямо в редакторе: гайд показывает приём, теория объясняет, а задание вы выполняете сами. Всё хранится на этом компьютере.
		</p>
		<label class="field">
			Как к вам обращаться?
			<!-- svelte-ignore a11y_autofocus -->
			<input type="text" bind:value={name} maxlength="40" placeholder="Имя" autofocus required />
		</label>
		<fieldset>
			<legend>Цель на день</legend>
			<div class="segmented">
				{#each [1, 2, 3, 5] as g (g)}
					<button type="button" aria-pressed={goal === g} onclick={() => (goal = g)}>{g} {g === 1 ? 'задание' : g < 5 ? 'задания' : 'заданий'}</button>
				{/each}
			</div>
		</fieldset>
		{#if error}<p class="error" role="alert">{error}</p>{/if}
		<button class="btn btn-accent" disabled={busy || !name.trim()}>Начать путь</button>
	</form>
</div>

<style>
	.welcome {
		position: fixed;
		inset: 0;
		z-index: 50;
		display: grid;
		place-items: center;
		background: var(--paper-base);
	}

	.enso {
		position: absolute;
		right: -120px;
		bottom: -140px;
	}

	form {
		position: relative;
		display: grid;
		gap: 20px;
		width: min(520px, calc(100vw - 48px));
		padding: 40px;
	}

	fieldset {
		display: grid;
		gap: 8px;
		margin: 0;
		padding: 0;
		border: 0;
	}

	legend {
		margin-bottom: 8px;
		font-size: 13px;
		font-weight: 600;
		color: var(--ink-700);
	}

	.error {
		color: var(--kaki-700);
	}

	.btn {
		justify-self: start;
	}
</style>
