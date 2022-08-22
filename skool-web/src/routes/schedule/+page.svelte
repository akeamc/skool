<script lang="ts">
	import { browser } from "$app/env";
	import { goto } from "$app/navigation";
	import { API_ENDPOINT } from "$lib/api";
	import { authenticated, sessionToken } from "$lib/auth";
	import Timetable from "$lib/timetable/Timetable.svelte";
	import type { TimetableType } from "$lib/schedule";

	$: {
		if (browser && !$authenticated) {
			goto("/login?next=/schedule");
		}
	}

	let cursor = browser ? new Date() : undefined;

	let timetablePromise: Promise<TimetableType> = new Promise(() => {});

	$: {
		if (!browser || !sessionToken) break $;

		timetablePromise = fetch(`${API_ENDPOINT}/schedule/timetables`, {
			headers: {
				authorization: `Bearer ${$sessionToken}`
			}
		}).then(async (res) => {
			if (!res.ok) {
				throw new Error(await res.text());
			}

			const data = await res.json();

			if (data.length !== 1) {
				throw new Error(`Expected 1 timetable, got ${data.length}`);
			}

			return data[0];
		});
	}
</script>

{#await timetablePromise}
	<p>loading</p>
{:then timetable}
	<Timetable {timetable} />
{:catch error}
	{error}
{/await}
