<script lang="ts">
  import { isAdminUser } from '$lib/auth';
  import { useAuth } from '$lib/use-auth.svelte';
  import MainApp from '$lib/components/main_app/MainApp.svelte';
  import SignIn from './SignIn.svelte';
  import * as Card from '$lib/components/ui/card/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { LoaderCircle } from '@lucide/svelte';

  const auth = useAuth();
</script>

<svelte:head>
  <title>Links Tool</title>
</svelte:head>

{#if auth.isPending || auth.isFetching}
  <Card.Root class="mx-auto max-w-md">
    <Card.Content class="flex flex-col items-center justify-center gap-3 py-16">
      <LoaderCircle class="text-muted-foreground size-8 animate-spin" aria-hidden="true" />
      <p class="text-muted-foreground text-sm" role="status">Checking session…</p>
    </Card.Content>
  </Card.Root>
{:else if auth.isError}
  <Card.Root class="mx-auto max-w-md">
    <Card.Header>
      <Card.Title>Could not load session</Card.Title>
      <Card.Description>
        {auth.error?.message ?? 'Something went wrong while checking your session.'}
      </Card.Description>
    </Card.Header>
    <Card.Content>
      <Button type="button" onclick={() => auth.refetch()}>Try again</Button>
    </Card.Content>
  </Card.Root>
{:else if isAdminUser(auth.data)}
  <MainApp />
{:else if auth.data}
  <Card.Root class="mx-auto max-w-md">
    <Card.Header>
      <Card.Title>Access denied</Card.Title>
      <Card.Description>
        Your account does not have permission to manage links. An admin role is required.
      </Card.Description>
    </Card.Header>
  </Card.Root>
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
