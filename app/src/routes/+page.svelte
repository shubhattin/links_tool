<script lang="ts">
  import { useAuth } from '$lib/use-auth.svelte';
  import MainApp from '$lib/components/main_app/MainApp.svelte';

  const auth = useAuth();
  import SignIn from './SignIn.svelte';
  import * as Card from '$lib/components/ui/card/index.js';
  import { LoaderCircle } from '@lucide/svelte';
</script>

<svelte:head>
  <title>Links Tool</title>
</svelte:head>

{#if auth.isPending}
  <Card.Root class="mx-auto max-w-md">
    <Card.Content class="flex flex-col items-center justify-center gap-3 py-16">
      <LoaderCircle class="text-muted-foreground size-8 animate-spin" aria-hidden="true" />
      <p class="text-muted-foreground text-sm" role="status">Checking session…</p>
    </Card.Content>
  </Card.Root>
{:else if auth.data}
  <MainApp />
{:else}
  <Card.Root class="mx-auto max-w-md">
    <!-- Currently disabled signup via email/password -->
    <!-- <Card.Header>
      <div class="flex gap-2">
        <Button
          variant={panel === 'sign-in' ? 'default' : 'ghost'}
          size="sm"
          type="button"
          aria-pressed={panel === 'sign-in'}
          onclick={() => switchPanel('sign-in')}>Sign in</Button
        >
        <Button
          variant={panel === 'sign-up' ? 'default' : 'ghost'}
          size="sm"
          type="button"
          aria-pressed={panel === 'sign-up'}
          onclick={() => switchPanel('sign-up')}>Sign up</Button
        >
      </div>
      <Card.Title>{panel === 'sign-in' ? 'Sign in' : 'Create an account'}</Card.Title>
      <Card.Description>
        {#if panel === 'sign-in'}
          Use your email and password to continue.
        {:else}
          Sign up with email and password. No verification step for now.
        {/if}
      </Card.Description>
    </Card.Header> -->
    <Card.Content>
      <!-- {#if panel === 'sign-in'}
      {:else}
      <SignUp />
      {/if} -->
      <SignIn />
    </Card.Content>
  </Card.Root>
{/if}
