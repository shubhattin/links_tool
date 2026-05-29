<script lang="ts">
  import { apiSignIn, apiSignUp } from '$lib/auth-api';
  import { auth, authLoading, completeAuth } from '$lib/auth';
  import { Button } from '$lib/components/ui/button/index.js';
  import * as Card from '$lib/components/ui/card/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Label } from '$lib/components/ui/label/index.js';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import { Separator } from '$lib/components/ui/separator/index.js';
  import { CheckCircle2, Loader2, XCircle } from '@lucide/svelte';

  type AuthPanel = 'sign-in' | 'sign-up';

  let panel = $state<AuthPanel>('sign-in');
  let email = $state('');
  let password = $state('');
  let name = $state('');
  let error = $state('');
  let loading = $state(false);

  function switchPanel(next: AuthPanel) {
    panel = next;
    error = '';
  }

  async function onSignIn(e: SubmitEvent) {
    e.preventDefault();
    error = '';
    loading = true;
    const result = await apiSignIn({ email, password });
    loading = false;
    if (!result.ok) {
      error = result.error;
      return;
    }
    const ok = await completeAuth(result.data);
    if (!ok) error = 'could not load session';
  }

  async function onSignUp(e: SubmitEvent) {
    e.preventDefault();
    error = '';
    loading = true;
    const result = await apiSignUp({ name, email, password });
    loading = false;
    if (!result.ok) {
      error = result.error;
      return;
    }
    const ok = await completeAuth(result.data);
    if (!ok) error = 'could not load session';
  }
</script>

<svelte:head>
  <title>Links Tool</title>
</svelte:head>

{#if $authLoading}
  <Card.Root class="mx-auto max-w-md">
    <Card.Content class="flex flex-col items-center justify-center gap-3 py-16">
      <Loader2 class="text-muted-foreground size-8 animate-spin" aria-hidden="true" />
      <p class="text-muted-foreground text-sm" role="status">Checking session…</p>
    </Card.Content>
  </Card.Root>
{:else if $auth.user}
  <Card.Root class="mx-auto max-w-md">
    <Card.Header>
      <Card.Title>Account</Card.Title>
      <Card.Description>You are signed in.</Card.Description>
    </Card.Header>
    <Card.Content class="flex flex-col gap-4">
      <div class="flex flex-col gap-1">
        <span class="text-muted-foreground text-xs font-medium uppercase tracking-wide">Name</span>
        <span class="text-base font-medium">{$auth.user.name}</span>
      </div>
      <Separator />
      <div class="flex flex-col gap-1">
        <span class="text-muted-foreground text-xs font-medium uppercase tracking-wide">Email</span>
        <span class="text-base">{$auth.user.email}</span>
      </div>
      <div class="flex flex-col gap-1">
        <span class="text-muted-foreground text-xs font-medium uppercase tracking-wide">Role</span>
        <Badge variant="secondary" class="w-fit capitalize">{$auth.user.role}</Badge>
      </div>
      <div class="flex flex-col gap-1">
        <span class="text-muted-foreground text-xs font-medium uppercase tracking-wide"
          >Email verified</span
        >
        <div class="flex items-center gap-2 text-sm">
          {#if $auth.user.email_verified}
            <CheckCircle2 class="text-primary size-4" />
            <span>Verified</span>
          {:else}
            <XCircle class="text-muted-foreground size-4" />
            <span class="text-muted-foreground">Not verified yet</span>
          {/if}
        </div>
      </div>
    </Card.Content>
  </Card.Root>
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
        <form class="flex flex-col gap-4" onsubmit={onSignIn}>
          <div class="flex flex-col gap-2">
            <Label for="email">Email</Label>
            <Input id="email" type="email" autocomplete="email" bind:value={email} required />
          </div>
          <div class="flex flex-col gap-2">
            <Label for="password">Password</Label>
            <Input
              id="password"
              type="password"
              autocomplete="current-password"
              bind:value={password}
              required
            />
          </div>
          {#if error}
            <p class="text-destructive text-sm" role="alert">{error}</p>
          {/if}
          <Button type="submit" class="w-full" disabled={loading}>
            {loading ? 'Signing in…' : 'Sign in'}
          </Button>
        </form>
      {:else}
        <form class="flex flex-col gap-4" onsubmit={onSignUp}>
          <div class="flex flex-col gap-2">
            <Label for="name">Name</Label>
            <Input id="name" type="text" autocomplete="name" bind:value={name} required />
          </div>
          <div class="flex flex-col gap-2">
            <Label for="signup-email">Email</Label>
            <Input
              id="signup-email"
              type="email"
              autocomplete="email"
              bind:value={email}
              required
            />
          </div>
          <div class="flex flex-col gap-2">
            <Label for="signup-password">Password</Label>
            <Input
              id="signup-password"
              type="password"
              autocomplete="new-password"
              minlength={8}
              bind:value={password}
              required
            />
            <p class="text-muted-foreground text-xs">At least 8 characters.</p>
          </div>
          {#if error}
            <p class="text-destructive text-sm" role="alert">{error}</p>
          {/if}
          <Button type="submit" class="w-full" disabled={loading}>
            {loading ? 'Creating account…' : 'Sign up'}
          </Button>
        </form>
      {/if}
    </Card.Content>
  </Card.Root>
{/if}
