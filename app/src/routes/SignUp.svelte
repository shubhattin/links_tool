<script lang="ts">
  import { apiSignUp } from '$lib/auth-api';
  import { completeAuth } from '$lib/auth';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Label } from '$lib/components/ui/label/index.js';

  let name = $state('');
  let email = $state('');
  let password = $state('');
  let error = $state('');
  let loading = $state(false);

  async function onSubmit(e: SubmitEvent) {
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

<form class="flex flex-col gap-4" onsubmit={onSubmit}>
  <div class="flex flex-col gap-2">
    <Label for="name">Name</Label>
    <Input id="name" type="text" autocomplete="name" bind:value={name} required />
  </div>
  <div class="flex flex-col gap-2">
    <Label for="signup-email">Email</Label>
    <Input id="signup-email" type="email" autocomplete="email" bind:value={email} required />
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
