<script lang="ts">
  import { onMount } from 'svelte';
  import { apiSignIn } from '$lib/auth-api';
  import { completeAuth } from '$lib/auth';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Label } from '$lib/components/ui/label/index.js';
  import { Separator } from '$lib/components/ui/separator/index.js';
  import { FaBrandsGoogle, FaBrandsGithub } from 'svelte-icons-pack/fa';
  import { Icon } from 'svelte-icons-pack';

  let email = $state('');
  let password = $state('');
  let error = $state('');
  let loading = $state(false);

  onMount(() => {
    const params = new URLSearchParams(window.location.search);
    const oauthError = params.get('error');
    if (oauthError) {
      error = oauthError.replaceAll('_', ' ');
      params.delete('error');
      const next = params.toString();
      const path = window.location.pathname;
      const url = next ? `${path}?${next}` : path;
      window.history.replaceState({}, '', url);
    }
  });

  async function onSubmit(e: SubmitEvent) {
    e.preventDefault();
    error = '';
    loading = true;
    try {
      const result = await apiSignIn({ email, password });
      if (!result.ok) {
        error = result.error;
        return;
      }
      const ok = await completeAuth(result.data);
      if (!ok) error = 'could not load session';
    } catch {
      error = 'request failed';
    } finally {
      loading = false;
    }
  }
</script>

<div class="flex flex-col gap-4">
  <Button
    variant="outline"
    class="w-full flex items-center gap-2 justify-center"
    href="/api/auth/google"
  >
    <Icon src={FaBrandsGoogle} className="size-4" />
    Continue with Google
  </Button>
  <Button
    variant="outline"
    class="w-full flex items-center gap-2 justify-center"
    href="/api/auth/github"
  >
    <Icon src={FaBrandsGithub} className="size-4" />
    Continue with GitHub
  </Button>

  <div class="relative">
    <Separator />
    <span
      class="bg-card text-muted-foreground absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 px-2 text-xs"
    >
      or
    </span>
  </div>

  <form class="flex flex-col gap-4" onsubmit={onSubmit}>
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
</div>
