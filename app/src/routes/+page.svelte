<script lang="ts">
  import { auth, authLoading } from '$lib/auth';
  import MainApp from './MainApp.svelte';
  import SignIn from './SignIn.svelte';
  import SignUp from './SignUp.svelte';
  import { Button } from '$lib/components/ui/button/index.js';
  import * as Card from '$lib/components/ui/card/index.js';
  import { LoaderCircle } from '@lucide/svelte';

  type AuthPanel = 'sign-in' | 'sign-up';

  let panel = $state<AuthPanel>('sign-in');

  function switchPanel(next: AuthPanel) {
    panel = next;
  }
</script>

<svelte:head>
  <title>Links Tool</title>
</svelte:head>

{#if $authLoading}
  <Card.Root class="mx-auto max-w-md">
    <Card.Content class="flex flex-col items-center justify-center gap-3 py-16">
      <LoaderCircle class="text-muted-foreground size-8 animate-spin" aria-hidden="true" />
      <p class="text-muted-foreground text-sm" role="status">Checking session…</p>
    </Card.Content>
  </Card.Root>
{:else if $auth.user}
  <MainApp />
{:else}
  <Card.Root class="mx-auto max-w-md">
    <Card.Header>
      <div class="flex gap-2">
        <Button
          variant={panel === 'sign-in' ? 'default' : 'ghost'}
          size="sm"
          type="button"
          onclick={() => switchPanel('sign-in')}>Sign in</Button
        >
        <Button
          variant={panel === 'sign-up' ? 'default' : 'ghost'}
          size="sm"
          type="button"
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
    </Card.Header>
    <Card.Content>
      {#if panel === 'sign-in'}
        <SignIn />
      {:else}
        <SignUp />
      {/if}
    </Card.Content>
  </Card.Root>
{/if}
