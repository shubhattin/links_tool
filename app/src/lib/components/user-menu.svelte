<script lang="ts">
  import { auth, signOut } from '$lib/auth.svelte';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import * as Popover from '$lib/components/ui/popover/index.js';
  import { Separator } from '$lib/components/ui/separator/index.js';
  import { CheckCircle2, User, XCircle } from '@lucide/svelte';

  async function handleSignOut() {
    await signOut();
  }
</script>

{#if $auth.user}
  <Popover.Root>
    <Popover.Trigger>
      {#snippet child({ props })}
        <Button {...props} variant="ghost" size="icon" aria-label="Account menu">
          <User data-icon="inline-start" />
        </Button>
      {/snippet}
    </Popover.Trigger>
    <Popover.Content align="end" class="w-72 gap-3 p-4">
      <div class="flex flex-col gap-1">
        <p class="text-muted-foreground text-xs font-medium uppercase tracking-wide">Name</p>
        <p class="font-medium leading-none">{$auth.user.name}</p>
      </div>
      <Separator />
      <div class="flex flex-col gap-1">
        <p class="text-muted-foreground text-xs font-medium uppercase tracking-wide">Email</p>
        <p class="text-sm break-all">{$auth.user.email}</p>
      </div>
      <div class="flex flex-col gap-1">
        <p class="text-muted-foreground text-xs font-medium uppercase tracking-wide">Role</p>
        <Badge variant="secondary" class="w-fit capitalize">{$auth.user.role}</Badge>
      </div>
      <div class="flex flex-col gap-1">
        <p class="text-muted-foreground text-xs font-medium uppercase tracking-wide">
          Email verified
        </p>
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
      <Separator />
      <Button variant="outline" class="w-full" onclick={handleSignOut}>Sign out</Button>
    </Popover.Content>
  </Popover.Root>
{/if}
