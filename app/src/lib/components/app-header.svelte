<script lang="ts">
  import ThemeToggle from '$lib/components/theme-toggle.svelte';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Separator } from '$lib/components/ui/separator/index.js';
  import { auth, signOut } from '$lib/auth';
  import { Link } from '@lucide/svelte';

  async function handleSignOut() {
    await signOut();
  }
</script>

<header class="border-border bg-background/95 supports-backdrop-filter:bg-background/60 sticky top-0 z-50 w-full border-b backdrop-blur">
  <div class="mx-auto flex h-14 max-w-5xl items-center gap-3 px-4">
    <a href="/" class="flex items-center gap-2 font-semibold tracking-tight">
      <Link class="size-5" />
      <span>Links Tool</span>
    </a>

    <div class="flex-1"></div>

    <nav class="flex items-center gap-2">
      {#if $auth.user}
        <span class="text-muted-foreground hidden text-sm sm:inline">{$auth.user.name}</span>
        <Button variant="outline" size="sm" onclick={handleSignOut}>Sign out</Button>
      {/if}
      <Separator orientation="vertical" class="mx-1 h-6" />
      <ThemeToggle />
    </nav>
  </div>
</header>
