import type { DateTime } from "luxon";

export interface TimetableType {
	school_guid: string;
	unit_guid: string;
	school_id: string;
	timetable_id: string | null;
	person_guid: string;
	first_name: string;
	last_name: string;
}

export interface LessonJson {
	teacher: string | null;
	location: string | null;
	start: string;
	end: string;
	course: string | null;
	id: string;
	color: string | null;
}

export interface LessonType extends Omit<LessonJson, "start" | "end"> {
	start: DateTime;
	end: DateTime;
}
