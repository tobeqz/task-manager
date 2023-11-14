//#[derive(Serialize, Deserialize, Debug)]
//pub struct Task {
//    pub title: String,
//    pub priority: Priority,
//    pub due: Option<DateTime<Utc>>,
//    pub sub: Vec<Subtask>,
//    pub done: bool,
//    pub done_at: Option<DateTime<Utc>>
//}

// TODO: Just use sveltekit damn

const BASE_URL = `http://${window.location.hostname}:4000`

export type Priority = "Low" | "Medium" | "High" | "Urgent";

interface TaskRaw {
	title: string,
	priority: Priority,
	due: string | null,
	sub: TaskRaw[],
	done: boolean,
	done_at: string | null
}

class Task {
	title: string
	priority: Priority
	due: Date | null
	sub: Task[]
	done: boolean
	done_at: Date | null

	constructor(raw: TaskRaw) {
		this.title = raw.title
		this.due = raw.due ? new Date(raw.due) : null 
		this.sub = raw.sub.map(subtask => new Task(subtask))
		this.done = raw.done
		this.done_at = raw.done_at ? new Date(raw.done_at) : null
	}
}

export async function getTasks() {
	const response = await fetch(`${BASE_URL}/tasks`)
	const tasks_raw: TaskRaw[] = await response.json()
	const tasks = tasks_raw.map(task => new Task(task))
	return tasks	
}
