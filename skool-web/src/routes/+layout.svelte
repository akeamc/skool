<script lang="ts">
import Header from "$lib/components/Header.svelte";

  import "../app.css";
</script>

<Header />

<main>
  <slot />
</main>

<style>
  main {
    padding-top: var(--header-height);
  }
</style>
