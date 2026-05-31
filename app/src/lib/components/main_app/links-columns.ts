import type { LinkDto } from '$lib/api';
import { renderComponent } from '$lib/components/ui/data-table/index.js';
import type { ColumnDef } from '@tanstack/table-core';
import LinkRowActions from './link-row-actions.svelte';
import LinkUrlCell from './link-url-cell.svelte';

export function createLinkColumns(handlers: {
  onEdit: (link: LinkDto) => void;
  onDelete: (link: LinkDto) => void;
}): ColumnDef<LinkDto>[] {
  return [
    {
      accessorKey: 'id',
      header: 'ID',
      cell: ({ row }) => row.original.id
    },
    {
      accessorKey: 'name',
      header: 'Name',
      cell: ({ row }) => row.original.name?.trim() || '—'
    },
    {
      accessorKey: 'link',
      header: 'Link',
      cell: ({ row }) => renderComponent(LinkUrlCell, { url: row.original.link })
    },
    {
      accessorKey: 'enabled',
      header: 'Status',
      cell: ({ row }) => (row.original.enabled ? 'Enabled' : 'Disabled')
    },
    {
      accessorKey: 'prefix_zeros',
      header: () => 'Prefix zeros',
      cell: ({ row }) => row.original.prefix_zeros
    },
    {
      id: 'actions',
      enableHiding: false,
      cell: ({ row }) =>
        renderComponent(LinkRowActions, {
          link: row.original,
          onEdit: handlers.onEdit,
          onDelete: handlers.onDelete
        })
    }
  ];
}
