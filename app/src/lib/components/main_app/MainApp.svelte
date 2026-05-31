<script lang="ts">
  import { client } from '$lib/api';
  import type { CreateLinkBody, LinkDto, LinksListResponse2, UpdateLinkBody } from '$lib/api';
  import { queryClient } from '$lib/query_client';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Label } from '$lib/components/ui/label/index.js';
  import * as Table from '$lib/components/ui/table/index.js';
  import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
  import * as Popover from '$lib/components/ui/popover/index.js';
  import * as AlertDialog from '$lib/components/ui/alert-dialog/index.js';
  import { Skeleton } from '$lib/components/ui/skeleton/index.js';
  import { FlexRender, createSvelteTable } from '$lib/components/ui/data-table/index.js';
  import { createLinkColumns } from './links-columns.js';
  import { linkRowKey } from './link-row-key.js';
  import { createMutation, createQuery } from '@tanstack/svelte-query';
  import {
    type SortingState,
    type VisibilityState,
    getCoreRowModel,
    getFilteredRowModel,
    getSortedRowModel
  } from '@tanstack/table-core';
  import { zErrorBody } from '$lib/api/generated/zod.gen';
  import { ChevronDown, LoaderCircle, RefreshCw } from '@lucide/svelte';
  import { toast } from 'svelte-sonner';

  const LINKS_QUERY_KEY = ['links'] as const;

  const linksQueryOptions = () => ({
    queryKey: LINKS_QUERY_KEY,
    queryFn: () => client.links.list()
  });

  const links_q = createQuery(linksQueryOptions, () => queryClient);

  let globalFilter = $state('');
  let sorting = $state<SortingState>([]);
  let columnVisibility = $state<VisibilityState>({});

  let formPopoverOpen = $state(false);
  let formMode = $state<'add' | 'edit'>('add');
  let editingId = $state<string | null>(null);
  let draftId = $state('');
  let draftName = $state('');
  let draftLink = $state('');
  let draftEnabled = $state(true);
  let draftPrefixZeros = $state(0);
  let formError = $state('');

  let saveDialogOpen = $state(false);
  let deleteDialogOpen = $state(false);
  let deleteTarget = $state<LinkDto | null>(null);

  const linkRows = $derived(links_q.data?.data?.links ?? []);
  const linksTableKey = $derived(linkRows.map(linkRowKey).join('\n'));

  function resetDraft() {
    draftId = '';
    draftName = '';
    draftLink = '';
    draftEnabled = true;
    draftPrefixZeros = 0;
    formError = '';
  }

  function openAddForm() {
    formMode = 'add';
    editingId = null;
    resetDraft();
    formPopoverOpen = true;
  }

  function openEditForm(link: LinkDto) {
    formMode = 'edit';
    editingId = link.id;
    draftId = link.id;
    draftName = link.name ?? '';
    draftLink = link.link;
    draftEnabled = link.enabled;
    draftPrefixZeros = link.prefix_zeros;
    formError = '';
    formPopoverOpen = true;
  }

  function openDeleteConfirm(link: LinkDto) {
    formPopoverOpen = false;
    deleteTarget = link;
    deleteDialogOpen = true;
  }

  function apiErrorMessage(error: unknown, fallback: string): string {
    const parsed = zErrorBody.safeParse(error);
    return parsed.success ? parsed.data.error : fallback;
  }

  type LinksListQueryData = {
    data?: LinksListResponse2;
    response?: Response;
    error?: unknown;
  };

  function patchLinksCache(updater: (links: LinkDto[]) => LinkDto[]) {
    queryClient.setQueryData<LinksListQueryData>(LINKS_QUERY_KEY, (old) => {
      if (!old?.data?.links) return old;
      return {
        ...old,
        data: {
          ...old.data,
          links: updater(old.data.links)
        }
      };
    });
  }

  async function refreshLinks() {
    await queryClient.refetchQueries({ queryKey: LINKS_QUERY_KEY, type: 'active' });
  }

  async function handleRefreshClick() {
    const result = await links_q.refetch();
    if (result.isError) {
      console.error(result.error);
      toast.error('Failed to refresh links');
      return;
    }
    toast.success('Links refreshed');
  }

  const create_mut = createMutation(
    () => ({
      mutationFn: async (body: CreateLinkBody) => {
        const result = await client.links.create({ body, throwOnError: false });
        if (result.error || !result.response?.ok) {
          throw new Error(apiErrorMessage(result.error, 'failed to create link'));
        }
        return result.data;
      },
      onSuccess: async (link) => {
        if (link) patchLinksCache((links) => [...links, link]);
        await refreshLinks();
        formPopoverOpen = false;
        saveDialogOpen = false;
        resetDraft();
        toast.success(`Created link "${link?.id ?? 'link'}"`);
      },
      onError: (err) => {
        toast.error(err instanceof Error ? err.message : 'Failed to create link');
      }
    }),
    () => queryClient
  );

  const update_mut = createMutation(
    () => ({
      mutationFn: async ({ id, body }: { id: string; body: UpdateLinkBody }) => {
        const result = await client.links.update({ path: { id }, body, throwOnError: false });
        if (result.error || !result.response?.ok) {
          throw new Error(apiErrorMessage(result.error, 'failed to update link'));
        }
        return result.data;
      },
      onSuccess: async (link, { id, body }) => {
        const updated: LinkDto = link ?? { id, ...body };
        patchLinksCache((links) => links.map((row) => (row.id === id ? updated : row)));
        await refreshLinks();
        formPopoverOpen = false;
        saveDialogOpen = false;
        resetDraft();
        toast.success(`Updated link "${id}"`);
      },
      onError: (err) => {
        toast.error(err instanceof Error ? err.message : 'Failed to update link');
      }
    }),
    () => queryClient
  );

  const delete_mut = createMutation(
    () => ({
      mutationFn: async (id: string) => {
        const result = await client.links.delete({ path: { id }, throwOnError: false });
        if (result.error || !result.response?.ok) {
          throw new Error(apiErrorMessage(result.error, 'failed to delete link'));
        }
        return id;
      },
      onSuccess: async (id) => {
        patchLinksCache((links) => links.filter((row) => row.id !== id));
        await refreshLinks();
        deleteDialogOpen = false;
        deleteTarget = null;
        toast.success(`Deleted link "${id}"`);
      },
      onError: (err) => {
        toast.error(err instanceof Error ? err.message : 'Failed to delete link');
      }
    }),
    () => queryClient
  );

  const linkColumns = $derived(
    createLinkColumns({
      onEdit: openEditForm,
      onDelete: openDeleteConfirm
    })
  );

  const table = createSvelteTable({
    get data() {
      return [...linkRows];
    },
    get columns() {
      return linkColumns;
    },
    state: {
      get globalFilter() {
        return globalFilter;
      },
      get sorting() {
        return sorting;
      },
      get columnVisibility() {
        return columnVisibility;
      }
    },
    onGlobalFilterChange: (updater) => {
      globalFilter = typeof updater === 'function' ? updater(globalFilter) : updater;
    },
    onSortingChange: (updater) => {
      sorting = typeof updater === 'function' ? updater(sorting) : updater;
    },
    onColumnVisibilityChange: (updater) => {
      columnVisibility = typeof updater === 'function' ? updater(columnVisibility) : updater;
    },
    getCoreRowModel: getCoreRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    getSortedRowModel: getSortedRowModel(),
    globalFilterFn: (row, _columnId, filterValue) => {
      const q = String(filterValue).toLowerCase().trim();
      if (!q) return true;
      const link = row.original;
      return (
        link.id.toLowerCase().includes(q) ||
        link.link.toLowerCase().includes(q) ||
        (link.name?.toLowerCase().includes(q) ?? false)
      );
    }
  });

  // TanStack Table caches row data internally; sync when the query list changes.
  $effect.pre(() => {
    const rows = linkRows;
    table.setOptions((prev) => ({ ...prev, data: [...rows] }));
  });

  const savePending = $derived(create_mut.isPending || update_mut.isPending);

  function requestSave() {
    formError = '';
    if (formMode === 'add' && !draftId.trim()) {
      formError = 'ID is required';
      return;
    }
    if (!draftLink.trim()) {
      formError = 'Link URL is required';
      return;
    }
    formPopoverOpen = false;
    saveDialogOpen = true;
  }

  async function confirmSave() {
    formError = '';
    const body = {
      enabled: draftEnabled,
      link: draftLink.trim(),
      prefix_zeros: Number(draftPrefixZeros) || 0,
      name: draftName.trim() ? draftName.trim() : null
    };

    try {
      if (formMode === 'add') {
        await create_mut.mutateAsync({
          ...body,
          id: draftId.trim()
        });
      } else if (editingId) {
        await update_mut.mutateAsync({ id: editingId, body });
      }
    } catch (err) {
      formError = err instanceof Error ? err.message : 'request failed';
      saveDialogOpen = false;
      formPopoverOpen = true;
    }
  }

  function cancelSaveDialog() {
    saveDialogOpen = false;
    formPopoverOpen = true;
  }

  async function confirmDelete() {
    if (!deleteTarget) return;
    try {
      await delete_mut.mutateAsync(deleteTarget.id);
    } catch (err) {
      formError = err instanceof Error ? err.message : 'request failed';
      deleteDialogOpen = false;
    }
  }
</script>

<div class="flex flex-col gap-6">
  <div class="flex flex-col gap-1">
    <h1 class="text-2xl font-semibold tracking-tight">Links</h1>
    <p class="text-muted-foreground text-sm">Manage short links, targets, and redirect settings.</p>
  </div>

  {#if links_q.error}
    <p class="text-destructive text-sm" role="alert">Error: {links_q.error.message}</p>
  {/if}

  {#if links_q.isPending}
    <div class="flex flex-col gap-4" role="status" aria-label="Loading links">
      <div class="flex flex-wrap items-center gap-2">
        <Skeleton class="h-9 max-w-sm flex-1" />
        <div class="ms-auto flex items-center gap-2">
          <Skeleton class="size-9 shrink-0" />
          <Skeleton class="h-9 w-24" />
          <Skeleton class="h-9 w-28" />
        </div>
      </div>
      <div class="rounded-md border">
        <Table.Root>
          <Table.Header>
            <Table.Row>
              {#each [72, 96, 160, 64, 48, 32] as width, i (i)}
                <Table.Head>
                  <Skeleton class="h-4" style="width: {width}px" />
                </Table.Head>
              {/each}
            </Table.Row>
          </Table.Header>
          <Table.Body>
            {#each Array(6) as _, rowIdx (rowIdx)}
              <Table.Row>
                {#each [72, 96, 160, 64, 48, 32] as width, colIdx (colIdx)}
                  <Table.Cell class="py-3">
                    <Skeleton class="h-4" style="width: {width}px; max-width: 100%" />
                  </Table.Cell>
                {/each}
              </Table.Row>
            {/each}
          </Table.Body>
        </Table.Root>
      </div>
    </div>
  {:else}
    <Popover.Root bind:open={formPopoverOpen}>
      <div class="flex flex-wrap items-center gap-2">
        <Input
          class="max-w-sm flex-1"
          placeholder="Filter links..."
          value={globalFilter}
          oninput={(e) => (globalFilter = e.currentTarget.value)}
        />
        <div class="ms-auto flex items-center gap-2">
          <Button
            variant="outline"
            size="icon"
            type="button"
            aria-label="Refresh links"
            disabled={links_q.isFetching}
            onclick={() => void handleRefreshClick()}
          >
            {#if links_q.isFetching}
              <LoaderCircle class="animate-spin" />
            {:else}
              <RefreshCw />
            {/if}
          </Button>
          <DropdownMenu.Root>
            <DropdownMenu.Trigger>
              {#snippet child({ props })}
                <Button {...props} variant="outline">
                  Columns
                  <ChevronDown data-icon="inline-end" />
                </Button>
              {/snippet}
            </DropdownMenu.Trigger>
            <DropdownMenu.Content align="end">
              {#each table.getAllColumns().filter((col) => col.getCanHide()) as column (column.id)}
                <DropdownMenu.CheckboxItem
                  class="capitalize"
                  checked={column.getIsVisible()}
                  onCheckedChange={(value) => column.toggleVisibility(!!value)}
                >
                  {column.id}
                </DropdownMenu.CheckboxItem>
              {/each}
            </DropdownMenu.Content>
          </DropdownMenu.Root>
          <Popover.Trigger>
            {#snippet child({ props })}
              <Button {...props} onclick={() => openAddForm()}>Add link</Button>
            {/snippet}
          </Popover.Trigger>
        </div>
      </div>

      <Popover.Content align="end" class="w-80">
        <div class="flex flex-col gap-4">
          <div class="flex flex-col gap-1">
            <h2 class="font-medium leading-none">
              {formMode === 'add' ? 'Add link' : 'Edit link'}
            </h2>
            <p class="text-muted-foreground text-xs">
              {formMode === 'add'
                ? 'Create a new short link entry.'
                : `Update settings for ${editingId}.`}
            </p>
          </div>

          <div class="flex flex-col gap-3">
            <div class="flex flex-col gap-1.5">
              <Label for="link-id">ID</Label>
              <Input
                id="link-id"
                disabled={formMode === 'edit'}
                bind:value={draftId}
                placeholder="short-id"
                maxlength={20}
              />
            </div>
            <div class="flex flex-col gap-1.5">
              <Label for="link-name">Name</Label>
              <Input
                id="link-name"
                bind:value={draftName}
                placeholder="Optional label"
                maxlength={30}
              />
            </div>
            <div class="flex flex-col gap-1.5">
              <Label for="link-url">Link</Label>
              <Input
                id="link-url"
                bind:value={draftLink}
                placeholder="https://example.com or path with {0}"
              />
            </div>
            <div class="flex flex-col gap-1.5">
              <Label for="link-prefix">Prefix zeros</Label>
              <Input id="link-prefix" type="number" min="0" bind:value={draftPrefixZeros} />
            </div>
            <label class="flex items-center gap-2 text-sm">
              <input type="checkbox" bind:checked={draftEnabled} class="size-4 rounded border" />
              Enabled
            </label>
          </div>

          {#if formError}
            <p class="text-destructive text-sm" role="alert">{formError}</p>
          {/if}

          <div class="flex justify-end gap-2">
            <Button variant="outline" type="button" onclick={() => (formPopoverOpen = false)}>
              Cancel
            </Button>
            <Button type="button" onclick={requestSave} disabled={savePending}>
              {#if savePending}
                <LoaderCircle class="animate-spin" data-icon="inline-start" />
              {/if}
              Save
            </Button>
          </div>
        </div>
      </Popover.Content>
    </Popover.Root>

    <div class="rounded-md border">
      <Table.Root>
        <Table.Header>
          {#each table.getHeaderGroups() as headerGroup (headerGroup.id)}
            <Table.Row>
              {#each headerGroup.headers as header (header.id)}
                <Table.Head>
                  {#if !header.isPlaceholder}
                    <FlexRender
                      content={header.column.columnDef.header}
                      context={header.getContext()}
                    />
                  {/if}
                </Table.Head>
              {/each}
            </Table.Row>
          {/each}
        </Table.Header>
        <Table.Body>
          {#key linksTableKey}
            {#each table.getRowModel().rows as row (linkRowKey(row.original))}
              <Table.Row>
                {#each row.getVisibleCells() as cell (`${linkRowKey(row.original)}-${cell.column.id}`)}
                  <Table.Cell>
                    <FlexRender content={cell.column.columnDef.cell} context={cell.getContext()} />
                  </Table.Cell>
                {/each}
              </Table.Row>
            {:else}
              <Table.Row>
                <Table.Cell colspan={linkColumns.length} class="h-24 text-center">
                  No links found.
                </Table.Cell>
              </Table.Row>
            {/each}
          {/key}
        </Table.Body>
      </Table.Root>
    </div>
  {/if}
</div>

<AlertDialog.Root bind:open={saveDialogOpen}>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>
        {formMode === 'add' ? 'Create link?' : 'Save changes?'}
      </AlertDialog.Title>
      <AlertDialog.Description>
        {#if formMode === 'add'}
          This will create short link <span class="font-mono">{draftId.trim()}</span>.
        {:else}
          This will update short link <span class="font-mono">{editingId}</span>.
        {/if}
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel onclick={cancelSaveDialog}>Cancel</AlertDialog.Cancel>
      <AlertDialog.Action onclick={confirmSave} disabled={savePending}>
        {#if savePending}
          <LoaderCircle class="animate-spin" data-icon="inline-start" />
        {/if}
        Confirm
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>

<AlertDialog.Root bind:open={deleteDialogOpen}>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>Delete link?</AlertDialog.Title>
      <AlertDialog.Description>
        {#if deleteTarget}
          This permanently deletes <span class="font-mono">{deleteTarget.id}</span>. This cannot be
          undone.
        {/if}
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
      <AlertDialog.Action
        variant="destructive"
        onclick={confirmDelete}
        disabled={delete_mut.isPending}
      >
        {#if delete_mut.isPending}
          <LoaderCircle class="animate-spin" data-icon="inline-start" />
        {/if}
        Delete
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
