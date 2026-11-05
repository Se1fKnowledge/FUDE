<script lang="ts">
	import type { Check, CheckKind, CheckProof, Range } from '../api.ts';
	import { CHECK_KINDS, TOOLS, defaultCheck } from '../checks.ts';
	import Icon from '../components/Icon.svelte';

	// One check of a step in the studio inspector: its kind, its numbers in plain words, and
	// what the live check found on the reference and on the untouched task.
	let {
		check,
		letters = [],
		colors = [],
		proof = null,
		selected = false,
		onchange,
		onremove,
		onfocus
	}: {
		check: Check;
		letters?: string[];
		colors?: string[];
		proof?: CheckProof | null;
		selected?: boolean;
		onchange: (check: Check) => void;
		onremove: () => void;
		onfocus?: () => void;
	} = $props();

	const c = $derived(check as Record<string, unknown>);
	const range = (key: string): Range => (c[key] as Range | undefined) ?? {};

	function set(key: string, v: unknown) {
		const next = { ...check } as Record<string, unknown>;
		if (v === undefined || v === '' || (typeof v === 'number' && Number.isNaN(v))) delete next[key];
		else next[key] = v;
		onchange(next as Check);
	}

	function setBound(key: string, side: 'min' | 'max', raw: string, scale = 1) {
		const r = { ...range(key) };
		if (raw.trim() === '') delete r[side];
		else r[side] = Math.round((Number(raw) / scale) * 1000) / 1000;
		set(key, r);
	}

	function setKind(kind: CheckKind['kind']) {
		onchange({ ...defaultCheck(kind), ...(check.message ? { message: check.message } : {}) });
	}

	const shown = (v: number | undefined, scale = 1) => (v === undefined ? '' : String(Math.round(v * scale * 10) / 10));

	/** Puts the suggested threshold into the check. */
	function apply(s: number) {
		if (check.kind === 'coverage') set('changed', { ...range('changed'), min: s });
		if (check.kind === 'darker') set('by', s);
		if (check.kind === 'similar') set('score', { ...range('score'), min: s });
	}

	const pct = (v: number | null | undefined) => (v === null || v === undefined ? '' : ` · ${Math.round(v * 100)}%`);
	const label = $derived(CHECK_KINDS.find((k) => k.kind === check.kind)?.label ?? check.kind);
	const verdictText = $derived(
		!proof
			? ''
			: proof.verdict === 'works'
				? 'Отличает сделанный шаг от несделанного.'
				: proof.verdict === 'passesUntouched'
					? 'Проходит и на нетронутом задании — ученик пройдёт шаг, ничего не сделав.'
					: proof.verdict === 'failsReference'
						? 'Не проходит даже на эталоне — шаг нельзя будет пройти.'
						: proof.verdict === 'noReference'
							? 'Загрузите эталон, чтобы проверить на нём.'
							: 'Смотрит на состояние редактора — проверяется в PhotoCraft, «Глазами ученика».'
	);
	const bad = $derived(proof?.verdict === 'passesUntouched' || proof?.verdict === 'failsReference');
</script>

<div class="card" class:bad class:on={selected} role="group" aria-label="Проверка: {label}">
	<div class="top">
		{#each letters as l, i (l)}
			<button class="tag" style:background={colors[i]} onclick={onfocus} aria-label="Показать область {l} на холсте">{l}</button>
		{/each}
		<select value={check.kind} onchange={(e) => setKind(e.currentTarget.value as CheckKind['kind'])} aria-label="Вид проверки">
			{#each CHECK_KINDS as k (k.kind)}<option value={k.kind}>{k.label}</option>{/each}
		</select>
		<button class="icon" onclick={onremove} aria-label="Удалить проверку"><Icon name="trash" size={15} /></button>
	</div>

	<div class="fields">
		{#if check.kind === 'tool'}
			<label>Инструмент
				<select value={check.tool} onchange={(e) => set('tool', e.currentTarget.value)}>
					{#each TOOLS as [id, name] (id)}<option value={id}>{name}</option>{/each}
				</select>
			</label>
		{:else if check.kind === 'brush'}
			{#each [['size', 'Размер, px', 1], ['hardness', 'Жёсткость, %', 100], ['opacity', 'Непрозрачность, %', 100]] as [key, name, scale] (key)}
				<div class="range"><span>{name}</span>
					<input type="number" placeholder="от" value={shown(range(String(key)).min, Number(scale))} onchange={(e) => setBound(String(key), 'min', e.currentTarget.value, Number(scale))} />
					<input type="number" placeholder="до" value={shown(range(String(key)).max, Number(scale))} onchange={(e) => setBound(String(key), 'max', e.currentTarget.value, Number(scale))} />
				</div>
			{/each}
		{:else if check.kind === 'zoom'}
			<div class="range"><span>Масштаб, %</span>
				<input type="number" placeholder="от" value={shown(range('zoom').min, 100)} onchange={(e) => setBound('zoom', 'min', e.currentTarget.value, 100)} />
				<input type="number" placeholder="до" value={shown(range('zoom').max, 100)} onchange={(e) => setBound('zoom', 'max', e.currentTarget.value, 100)} />
			</div>
		{:else if check.kind === 'layers'}
			<div class="range"><span>Слоёв</span>
				<input type="number" placeholder="от" value={shown(range('count').min)} onchange={(e) => setBound('count', 'min', e.currentTarget.value)} />
				<input type="number" placeholder="до" value={shown(range('count').max)} onchange={(e) => setBound('count', 'max', e.currentTarget.value)} />
			</div>
		{:else if check.kind === 'layer'}
			<label>Имя содержит <input type="text" value={check.name ?? ''} onchange={(e) => set('name', e.currentTarget.value.trim())} /></label>
			<label class="inline"><input type="checkbox" checked={check.visible === false} onchange={(e) => set('visible', e.currentTarget.checked ? false : undefined)} /> Слой скрыт</label>
		{:else if check.kind === 'selection'}
			<label class="inline"><input type="checkbox" checked={check.present !== false} onchange={(e) => set('present', e.currentTarget.checked)} /> Выделение есть</label>
		{:else if check.kind === 'history'}
			<label class="inline"><input type="checkbox" checked={check.can_redo === true} onchange={(e) => set('can_redo', e.currentTarget.checked || undefined)} /> Последнее действие отменено</label>
		{:else if check.kind === 'foreground' || check.kind === 'color'}
			<label>Цвет <input type="color" value={check.color} onchange={(e) => set('color', e.currentTarget.value.toUpperCase())} /></label>
		{:else if check.kind === 'coverage'}
			<p class="sentence">В области <b style:color={colors[0]}>{letters[0]}</b> изменилось не меньше
				<input type="number" min="1" max="100" value={shown(range('changed').min, 100)} onchange={(e) => setBound('changed', 'min', e.currentTarget.value, 100)} /> %</p>
		{:else if check.kind === 'luminance'}
			<div class="range"><span>Яркость <b style:color={colors[0]}>{letters[0]}</b>, %</span>
				<input type="number" placeholder="от" value={shown(range('value').min, 100)} onchange={(e) => setBound('value', 'min', e.currentTarget.value, 100)} />
				<input type="number" placeholder="до" value={shown(range('value').max, 100)} onchange={(e) => setBound('value', 'max', e.currentTarget.value, 100)} />
			</div>
		{:else if check.kind === 'darker'}
			<p class="sentence"><b style:color={colors[0]}>{letters[0]}</b> темнее, чем <b style:color={colors[1]}>{letters[1]}</b>, на
				<input type="number" min="1" max="100" value={shown(check.by ?? 0.05, 100)} onchange={(e) => set('by', Number(e.currentTarget.value) / 100)} /> %</p>
		{:else if check.kind === 'similar'}
			<p class="sentence">Похоже на эталон не меньше чем на
				<input type="number" min="1" max="100" value={shown(range('score').min, 100)} onchange={(e) => setBound('score', 'min', e.currentTarget.value, 100)} /> %</p>
		{/if}
		<label>Сообщение ученику <input type="text" value={check.message ?? ''} placeholder="Составится само" onchange={(e) => set('message', e.currentTarget.value.trim())} /></label>
	</div>

	{#if proof}
		{#if proof.canvas}
			<div class="pills">
				<span class="pill" class:ok={proof.onReference?.passed} class:no={proof.onReference && !proof.onReference.passed}>
					эталон: {proof.onReference ? (proof.onReference.passed ? 'проходит' : 'не проходит') : '—'}{pct(proof.onReference?.value)}
				</span>
				<span class="pill" class:ok={proof.onTask && !proof.onTask.passed} class:no={proof.onTask?.passed}>
					пустое: {proof.onTask ? (proof.onTask.passed ? 'тоже проходит' : 'не проходит') : '—'}{pct(proof.onTask?.value)}
				</span>
			</div>
		{/if}
		<p class="verdict" class:warn={bad}>{verdictText}</p>
		{#if proof.suggestion !== null}
			<button class="apply" onclick={() => apply(proof!.suggestion!)}>Поставить порог {Math.round(proof.suggestion * 100)}%</button>
		{/if}
	{/if}
</div>

<style>
	.card {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 12px;
		border: 1px solid transparent;
		border-radius: 8px;
		background: var(--sumi-raised);
	}

	.card.on {
		border-color: var(--ai-300);
	}

	.card.bad {
		border-color: var(--kaki-500);
	}

	.top {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.tag {
		display: grid;
		place-items: center;
		min-width: 22px;
		height: 22px;
		border: 0;
		border-radius: 6px;
		color: #fff;
		font: 700 12px var(--font-ui);
		cursor: pointer;
	}

	select,
	input[type='text'],
	input[type='number'] {
		min-width: 0;
		padding: 6px 8px;
		border: 1px solid var(--sumi-line);
		border-radius: 6px;
		background: var(--sumi-bg);
		color: var(--sumi-text);
		font: 13px var(--font-ui);
	}

	.top select {
		flex: 1;
		font-weight: 600;
	}

	.icon {
		display: grid;
		place-items: center;
		width: 28px;
		height: 28px;
		border: 0;
		border-radius: 6px;
		background: none;
		color: var(--sumi-muted);
		cursor: pointer;
	}

	.icon:hover {
		color: var(--sumi-text);
		background: var(--sumi-line);
	}

	.fields {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	label {
		display: flex;
		flex-direction: column;
		gap: 4px;
		color: var(--sumi-muted);
		font: 600 12px var(--font-ui);
	}

	label.inline {
		flex-direction: row;
		align-items: center;
		gap: 8px;
		color: var(--sumi-text);
	}

	input[type='color'] {
		width: 48px;
		height: 28px;
		border: 1px solid var(--sumi-line);
		border-radius: 6px;
		background: none;
	}

	.range {
		display: grid;
		grid-template-columns: 1fr 64px 64px;
		gap: 6px;
		align-items: center;
		color: var(--sumi-muted);
		font: 600 12px var(--font-ui);
	}

	.sentence {
		margin: 0;
		color: var(--sumi-text);
		font: 13px/2 var(--font-ui);
	}

	.sentence input {
		width: 60px;
	}

	.pills {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}

	.pill {
		padding: 3px 8px;
		border-radius: 999px;
		background: var(--sumi-line);
		color: var(--sumi-muted);
		font: 600 12px var(--font-ui);
	}

	.pill.ok {
		background: var(--ai-900);
		color: var(--ai-100);
	}

	.pill.no {
		background: var(--shu-700);
		color: var(--shu-100);
	}

	.verdict {
		margin: 0;
		color: var(--sumi-muted);
		font: 12px/1.45 var(--font-ui);
	}

	.verdict.warn {
		color: var(--kaki-500);
	}

	.apply {
		align-self: flex-start;
		padding: 7px 12px;
		border: 0;
		border-radius: 8px;
		background: var(--kaki-500);
		color: var(--ink-900);
		font: 600 13px var(--font-ui);
		cursor: pointer;
	}
</style>
