<script lang="ts">
  import '../app.css';
  import { dev } from '$app/environment';
  import AuthInit from '$lib/components/auth-init.svelte';
  import AppHeader from '$lib/components/app-header.svelte';
  import { ModeWatcher } from 'mode-watcher';
  import { QueryClientProvider } from '@tanstack/svelte-query';
  import { SvelteQueryDevtools } from '@tanstack/svelte-query-devtools';
  import { queryClient } from '$lib/query_client';
  import { Toaster } from '$lib/components/ui/sonner/index.js';
</script>

<ModeWatcher />

<QueryClientProvider client={queryClient}>
  <AuthInit>
    <div class="bg-background text-foreground flex min-h-svh flex-col">
      <AppHeader />
      <main class="mx-auto w-full max-w-5xl flex-1 px-4 py-8">
        <slot />
      </main>
    </div>
    <Toaster richColors closeButton position="top-right" />
  </AuthInit>
  {#if dev}
    <SvelteQueryDevtools />
  {/if}
</QueryClientProvider>
