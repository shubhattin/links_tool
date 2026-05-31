<script lang="ts">
  import type { LinkDto } from '$lib/api';
  import { Button } from '$lib/components/ui/button/index.js';
  import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
  import { Ellipsis } from '@lucide/svelte';

  let {
    link,
    onEdit,
    onDelete
  }: {
    link: LinkDto;
    onEdit: (link: LinkDto) => void;
    onDelete: (link: LinkDto) => void;
  } = $props();
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger>
    {#snippet child({ props })}
      <Button {...props} variant="ghost" size="icon-sm" aria-label="Row actions">
        <Ellipsis />
      </Button>
    {/snippet}
  </DropdownMenu.Trigger>
  <DropdownMenu.Content align="end" class="w-40">
    <DropdownMenu.Item onclick={() => onEdit(link)}>Edit</DropdownMenu.Item>
    <DropdownMenu.Item variant="destructive" onclick={() => onDelete(link)}>
      Delete
    </DropdownMenu.Item>
  </DropdownMenu.Content>
</DropdownMenu.Root>
