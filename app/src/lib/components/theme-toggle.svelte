<script lang="ts">
  import { Button } from '$lib/components/ui/button/index.js';
  import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
  import { Monitor, Moon, Sun } from '@lucide/svelte';
  import { mode, setMode } from 'mode-watcher';

  const currentMode = $derived(mode.current);
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger>
    {#snippet child({ props })}
      <Button {...props} variant="ghost" size="icon" aria-label="Toggle theme">
        {#if currentMode === 'dark'}
          <Moon data-icon="inline-start" />
        {:else if currentMode === 'light'}
          <Sun data-icon="inline-start" />
        {:else}
          <Monitor data-icon="inline-start" />
        {/if}
      </Button>
    {/snippet}
  </DropdownMenu.Trigger>
  <DropdownMenu.Content align="end">
    <DropdownMenu.Item onclick={() => setMode('light')}>
      <Sun data-icon="inline-start" />
      Light
    </DropdownMenu.Item>
    <DropdownMenu.Item onclick={() => setMode('dark')}>
      <Moon data-icon="inline-start" />
      Dark
    </DropdownMenu.Item>
    <DropdownMenu.Item onclick={() => setMode('system')}>
      <Monitor data-icon="inline-start" />
      System
    </DropdownMenu.Item>
  </DropdownMenu.Content>
</DropdownMenu.Root>
