<script lang="ts">
	import { authenticated, logout } from "./auth";
	import Button from "./Button.svelte";

	let scrollY: number;
</script>

<header class:floating={scrollY > 0}>
	<nav>
		<ul>
			<li><a href="/">Start</a></li>
			<li><a href="/schedule">Schema</a></li>
		</ul>
	</nav>
	<div class="auth">
		{#if $authenticated}
			<Button type="button" on:click={logout}>Logga ut</Button>
		{:else}
			<a href="/login">
				<Button type="button">Logga in</Button>
			</a>
		{/if}
	</div>
</header>

<svelte:window bind:scrollY />

<style>
	header {
		--border-radius: 16px;

		height: var(--header-height);
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		z-index: 100000;
		transition: all 0.1s;
		display: flex;
		justify-content: space-between;
		gap: 24px;
		padding-inline: 24px;
	}

	header::after {
		content: "";
		position: absolute;
		bottom: 0;
		left: 0;
		right: 0;
		height: 1px;
		background-color: #ddd;
		transition: inherit;
	}

	header.floating {
		background-color: var(--background-primary);
		border-radius: 0 0 var(--border-radius) var(--border-radius);
		border: 0;
	}

	header.floating::after {
		opacity: 0;
	}

	nav ul {
		--pad-x: 16px;

		list-style: none;
		padding: 0;
		margin: 0;
		display: flex;
		align-items: center;
		max-width: calc(var(--content-width) + 2 * var(--pad-x));
		box-sizing: content-box;
	}

	nav a {
		text-decoration: none;
		color: var(--text-primary);
		font-family: var(--font-main);
		font-weight: 600;
		font-size: 14px;
		height: var(--header-height);
		display: inline-flex;
		align-items: center;
		padding: 0 var(--pad-x);
		letter-spacing: -0.006em;
		position: relative;
	}

	nav a::after {
		height: 2px;
		width: calc(100% - 2 * var(--pad-x));
		background-color: var(--brand-primary);
		content: "";
		position: absolute;
		bottom: 0;
		transform: scaleX(0);
		opacity: 0;
		transition: all 0.15s;
		z-index: 1;
	}

	nav a:hover::after {
		transform: scaleX(1);
		opacity: 1;
	}

	nav a:focus-visible {
		outline: var(--brand-primary) solid 2px;
	}

	.auth {
		display: flex;
		align-items: center;
	}
</style>
