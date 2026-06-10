<script lang="ts">
  import { createQuery } from '@tanstack/svelte-query';
  import { setContext, type Snippet } from 'svelte';
  import { authQueryOptions, scheduleProactiveRefresh } from '$lib/auth';
  import { queryClient } from '$lib/query_client';
  import { AUTH_QUERY_CONTEXT } from '$lib/use-auth.svelte';

  let { children }: { children: Snippet } = $props();

  const auth = createQuery(authQueryOptions, () => queryClient);
  setContext(AUTH_QUERY_CONTEXT, auth);

  let proactiveRefreshBootstrapped = false;

  $effect(() => {
    if (proactiveRefreshBootstrapped || auth.isPending || auth.isFetching) return;
    proactiveRefreshBootstrapped = true;
    if (auth.data) scheduleProactiveRefresh();
  });

  $effect(() => {
    if (!auth.isError) return;
    proactiveRefreshBootstrapped = false;
  });
</script>

{@render children()}
