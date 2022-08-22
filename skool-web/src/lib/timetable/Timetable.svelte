<script lang="ts">
	import { browser } from "$app/env";
	import { DateTime } from "luxon";
	import { API_ENDPOINT } from "$lib/api";
	import { sessionToken } from "$lib/auth";
	import type { TimetableType, LessonJson, LessonType } from "$lib/schedule";

	export let timetable: TimetableType;

	type Scope = "day" | "week";

	let lessons: Promise<LessonType[]> = new Promise(() => {});
	let cursor = browser ? DateTime.now() : undefined;
	let scope: Scope = "week";

	$: year = cursor?.year;
	$: week = cursor?.weekNumber;

	$: {
		if (!browser || !$sessionToken || !timetable?.timetable_id || !year || !week) break $;

		lessons = fetch(
			`${API_ENDPOINT}/schedule/timetables/${timetable.timetable_id}/lessons?year=${year}&week=${week}`,
			{
				headers: {
					authorization: `Bearer ${$sessionToken}`
				}
			}
		).then(async (res) => {
			if (!res.ok) {
				throw new Error(await res.text());
			}

			let lessons: LessonType[] = ((await res.json()) as LessonJson[]).map((l) => ({
				...l,
				start: DateTime.fromISO(l.start),
				end: DateTime.fromISO(l.end)
			}));

			lessons.sort((a, b) => +a.start - +b.start);

			return lessons;
		});
	}
</script>

<h1>
	{cursor?.startOf(scope).toLocaleString(DateTime.DATE_MED)} – {cursor
		?.endOf(scope)
		.toLocaleString(DateTime.DATE_MED)}
</h1>

{#await lessons}
	<p>loading</p>
{:then lessons}
	<pre>{JSON.stringify(lessons, null, 2)}</pre>
{:catch error}
	{error}
{/await}
