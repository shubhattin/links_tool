<script lang="ts">
  import '../app.css';
  import AppHeader from '$lib/components/app-header.svelte';
  import { ModeWatcher } from 'mode-watcher';
  import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query';
  import { onMount } from 'svelte';
  import { startAuthRefreshLoop } from '$lib/auth';
  import ms from 'ms';

  const STALE_TIME_MS = ms('15mins'); // by default data will stay fresh for 8 minutes

  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        staleTime: STALE_TIME_MS
      }
    }
  });

  onMount(() => {
    startAuthRefreshLoop();
  });
</script>

<ModeWatcher />

<QueryClientProvider client={queryClient}>
  <div class="bg-background text-foreground flex min-h-svh flex-col">
    <AppHeader />
    <main class="mx-auto w-full max-w-5xl flex-1 px-4 py-8">
      <slot />
    </main>
  </div>
</QueryClientProvider>
