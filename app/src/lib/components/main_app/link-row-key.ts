import type { LinkDto } from '$lib/api';

/** Stable row identity for TanStack Table + Svelte `{#each}` when link fields change. */
export function linkRowKey(link: LinkDto): string {
  return `${link.id}|${link.link}|${link.name ?? ''}|${link.enabled}|${link.prefix_zeros}`;
}
