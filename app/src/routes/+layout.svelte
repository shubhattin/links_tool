<script lang="ts">
  import '../app.css';
  import AppHeader from '$lib/components/app-header.svelte';
  import { ModeWatcher } from 'mode-watcher';
  import { onMount } from 'svelte';
  import { startAuthRefreshLoop } from '$lib/auth';

  onMount(() => {
    startAuthRefreshLoop();
  });
</script>

<ModeWatcher />

<div class="bg-background text-foreground flex min-h-svh flex-col">
  <AppHeader />
  <main class="mx-auto w-full max-w-5xl flex-1 px-4 py-8">
    <slot />
  </main>
</div>
