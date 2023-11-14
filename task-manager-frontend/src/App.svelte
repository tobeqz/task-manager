<script lang="ts">
    import { onMount } from 'svelte';
  	import svelteLogo from './assets/svelte.svg'
  	import viteLogo from '/vite.svg'
	import { getTasks } from "./backend/task"

  	// Is the user logged in
	onMount(() => {
		getTasks()
			.then(tasks => {
				console.log(tasks)
			})
			.catch(error => {
				alert("Something went wrong")
				console.error(error)
			})
	})
</script>

<main>
	Hi
</main>

