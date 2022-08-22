<script lang="ts">
	import Button from "$lib/Button.svelte";
	import { authenticated, authenticating, createSession } from "$lib/auth";
	import { goto } from "$app/navigation";
	import { page } from "$app/stores";

	const handleSubmit: svelte.JSX.EventHandler<SubmitEvent, HTMLFormElement> = async ({
		target
	}) => {
		const data = new FormData(target as any);

		const username = data.get("username")?.toString();
		const password = data.get("password")?.toString();

		if (!username || !password) {
			return;
		}

		await createSession({ username, password });
	};

	$: {
		if ($authenticated) {
			goto($page.url.searchParams.get("next") || "/");
		}
	}
</script>

<div class="root">
	<div class="card">
		{#if $authenticated}
			<p>Du är redan inloggad; omdirigerar ...</p>
		{:else}
			<h1>Logga in</h1>
			<p>Dina inloggningsuppgifter krypteras och sparas bara på den här enheten.</p>
			<form on:submit|preventDefault={handleSubmit}>
				<label for="username">Användarnamn</label>
				<input id="username" name="username" placeholder="ab12345" />

				<label for="password">Lösenord</label>
				<input id="password" name="password" type="password" />

				<Button type="submit" disabled={$authenticating}>
					{$authenticating ? "Loggar in ..." : "Logga in"}
				</Button>
			</form>
		{/if}
	</div>
</div>

<style>
	.root {
		display: flex;
		flex-direction: column;
		align-items: center;
		min-height: calc(100vh - var(--header-height));
	}

	.card {
		padding: 0 var(--page-gutter);
		margin: calc(var(--header-height) + var(--page-gutter)) 0;
		text-align: center;
	}

	@media (min-width: 480px) {
		.root {
			background-color: var(--background-secondary);
			justify-content: center;
		}

		.card {
			padding: var(--page-gutter);
			width: 400px;
			border-radius: 12px;
			box-shadow: 0px 1px 1px rgba(0, 0, 0, 0.1);
			background-color: var(--background-primary);
		}
	}

	h1 {
		font-size: 2rem;
		font-weight: 500;
		letter-spacing: -0.025em;
		margin: 1rem 0;
	}

	p {
		color: var(--text-muted);
		font-size: 0.875rem;
		line-height: 1.5;
		letter-spacing: -0.006em;
		margin: 1rem 0;
		font-weight: 400;
	}

	form {
		text-align: start;
	}

	input {
		height: 3rem;
		border: 1px solid #eee;
		border-radius: 4px;
		width: 100%;
		margin-top: 0.5rem;
		margin-bottom: 24px;
		box-sizing: border-box;
		padding-inline: 8px;
		font-family: var(--font-main);
		font-size: 14px;
	}

	input::placeholder {
		color: #aaa;
		opacity: 1;
	}

	input:focus {
		outline: var(--brand-primary) solid 2px;
	}

	label {
		font-family: var(--font-main);
		font-size: 14px;
		font-weight: 500;
	}
</style>
