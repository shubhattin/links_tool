import { QueryClient } from '@tanstack/svelte-query';
import ms from 'ms';

export const DEFAULT_STALE_TIME_MS = ms('15mins');
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: DEFAULT_STALE_TIME_MS
    }
  }
});
