<script lang="ts">
	import { onMount } from 'svelte';
	import Enso from './Enso.svelte';
	import Hanko from './Hanko.svelte';

	// The opening animation from the mockup (Figma «03 · Анимация открытия», 3.6 s): a drop
	// of ink falls, spreads over the paper, the brush paints an ensō, 筆 emerges, the name
	// appears and the 学 seal strikes. With reduced motion it is a 0.6 s fade.
	let { ondone }: { ondone: () => void } = $props();

	let leaving = $state(false);

	onMount(() => {
		const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
		const total = reduced ? 600 : 3050;
		const leave = setTimeout(() => (leaving = true), total);
		const done = setTimeout(ondone, total + 550);
		return () => {
			clearTimeout(leave);
			clearTimeout(done);
		};
	});
</script>

<div class="splash" class:leaving role="presentation" onclick={() => ((leaving = true), setTimeout(ondone, 300))}>
	<div class="stage">
		<span class="drop"></span>
		<span class="wash"></span>
		<div class="enso"><Enso size={300} width={22} draw duration={880} delay={620} seed={5} /></div>
		<span class="fude kanji">筆</span>
		<div class="title">
			<span class="name">FUDE</span>
			<span class="sub">школа цифрового рисунка</span>
		</div>
		<div class="seal"><Hanko kanji="学" size={54} tilt={-8} /></div>
	</div>
</div>

<style>
	.splash {
		position: fixed;
		inset: 0;
		z-index: 100;
		display: grid;
		place-items: center;
		background: var(--paper-base);
		transition:
			opacity 520ms var(--ease-out),
			transform 520ms var(--ease-out);
		cursor: default;
	}

	.leaving {
		opacity: 0;
		transform: scale(1.04);
		pointer-events: none;
	}

	.stage {
		position: relative;
		width: 300px;
		height: 300px;
	}

	.drop {
		position: absolute;
		left: 50%;
		top: 50%;
		width: 14px;
		height: 18px;
		margin: -9px 0 0 -7px;
		border-radius: 50% 50% 50% 50% / 60% 60% 40% 40%;
		background: var(--ink-900);
		animation: drop 420ms ease-in both;
	}

	.wash {
		position: absolute;
		inset: 0;
		border-radius: 50%;
		background: radial-gradient(circle, rgb(47 93 140 / 18%) 0, rgb(47 93 140 / 8%) 45%, transparent 70%);
		animation: wash 1080ms cubic-bezier(0.16, 1, 0.3, 1) 420ms both;
	}

	.enso {
		position: absolute;
		inset: 0;
	}

	.fude {
		position: absolute;
		inset: 0;
		display: grid;
		place-items: center;
		font-size: 120px;
		color: var(--ink-900);
		animation: emerge 600ms ease-out 1500ms both;
	}

	.title {
		position: absolute;
		left: 50%;
		top: 100%;
		display: grid;
		justify-items: center;
		gap: 4px;
		margin-top: 28px;
		transform: translateX(-50%);
		white-space: nowrap;
		animation: rise 720ms ease-out 2180ms both;
	}

	.name {
		font-family: var(--font-display);
		font-weight: 600;
		font-size: 44px;
		letter-spacing: 0.3em;
		margin-right: -0.3em;
	}

	.sub {
		font-size: 14px;
		color: var(--ink-500);
	}

	.seal {
		position: absolute;
		right: 2px;
		bottom: 18px;
		animation: seal 450ms cubic-bezier(0.3, 1.6, 0.5, 1) 2600ms both;
	}

	@keyframes drop {
		from {
			transform: translateY(-140px) scaleY(1.3);
			opacity: 0;
		}
		80% {
			opacity: 1;
		}
		to {
			transform: translateY(0) scale(0.2);
			opacity: 0;
		}
	}

	@keyframes wash {
		from {
			transform: scale(0.05);
			opacity: 1;
		}
		to {
			transform: scale(1.25);
			opacity: 1;
		}
	}

	@keyframes emerge {
		from {
			opacity: 0;
			filter: blur(22px);
			transform: scale(1.08);
		}
	}

	@keyframes rise {
		from {
			opacity: 0;
			transform: translate(-50%, 10px);
		}
	}

	@keyframes seal {
		from {
			opacity: 0;
			transform: rotate(-30deg) scale(2.2);
		}
	}
</style>
