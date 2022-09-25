<script lang="ts">
	import '../app.css';
	import { Styles } from 'sveltestrap';
</script>

<Styles />
<main>
	<slot />
</main>

<style>
	main {
		padding: 1rem;
		width: 100%;
		margin: 0 auto;
	}
</style>
