<script lang="ts">
  import { useAuth } from '$lib/use-auth.svelte';

  const auth = useAuth();
  import { Badge } from '$lib/components/ui/badge/index.js';
  import * as Card from '$lib/components/ui/card/index.js';
  import { Separator } from '$lib/components/ui/separator/index.js';
  import { CircleCheck, CircleX } from '@lucide/svelte';
</script>

<Card.Root class="mx-auto max-w-md">
  <Card.Header>
    <Card.Title>Account</Card.Title>
    <Card.Description>You are signed in.</Card.Description>
  </Card.Header>
  <Card.Content class="flex flex-col gap-4">
    <div class="flex flex-col gap-1">
      <span class="text-muted-foreground text-xs font-medium uppercase tracking-wide">Name</span>
      <span class="text-base font-medium">{auth.data?.name}</span>
    </div>
    <Separator />
    <div class="flex flex-col gap-1">
      <span class="text-muted-foreground text-xs font-medium uppercase tracking-wide">Email</span>
      <span class="text-base">{auth.data?.email}</span>
    </div>
    <div class="flex flex-col gap-1">
      <span class="text-muted-foreground text-xs font-medium uppercase tracking-wide">Role</span>
      <Badge variant="secondary" class="w-fit capitalize">{auth.data?.role}</Badge>
    </div>
    <div class="flex flex-col gap-1">
      <span class="text-muted-foreground text-xs font-medium uppercase tracking-wide"
        >Email verified</span
      >
      <div class="flex items-center gap-2 text-sm">
        {#if auth.data?.email_verified}
          <CircleCheck class="text-primary size-4" />
          <span>Verified</span>
        {:else}
          <CircleX class="text-muted-foreground size-4" />
          <span class="text-muted-foreground">Not verified yet</span>
        {/if}
      </div>
    </div>
  </Card.Content>
</Card.Root>
