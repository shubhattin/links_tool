import { getContext } from 'svelte';
import type { CreateQueryResult } from '@tanstack/svelte-query';
import type { AuthUser } from './auth';

export const AUTH_QUERY_CONTEXT = Symbol('auth-query');

export type AuthQuery = CreateQueryResult<AuthUser | null, Error>;

export function useAuth(): AuthQuery {
  return getContext<AuthQuery>(AUTH_QUERY_CONTEXT);
}
