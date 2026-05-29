<script lang="ts">
  import { apiSignIn } from '$lib/auth-api';
  import { completeAuth } from '$lib/auth';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Label } from '$lib/components/ui/label/index.js';

  let email = $state('');
  let password = $state('');
  let error = $state('');
  let loading = $state(false);

  async function onSubmit(e: SubmitEvent) {
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
</script>

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
