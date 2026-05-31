<script lang="ts">
  import { client } from '$lib/api';
  import { createQuery } from '@tanstack/svelte-query';

  const links_q = createQuery(() => ({ queryKey: ['links'], queryFn: () => client.links.list() }));
</script>

{#if links_q.isPending}
  <div>Loading...</div>
{:else if links_q.error}
  <div>Error: {links_q.error.message}</div>
{:else if links_q.isSuccess}
  <div>
    <h1>Links</h1>
    <ul>
      {#each links_q.data.data?.links ?? [] as link}
        <li>{link.name}</li>
      {/each}
    </ul>
  </div>
{/if}
