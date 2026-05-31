/**
 * Runtime config for the generated Hey API ky client.
 * Session refresh on 401 is handled in `ky` (with per-route opt-out).
 */
import type { CreateClientConfig } from './generated/client.gen';
import { api } from '$lib/api/ky';

export const createClientConfig: CreateClientConfig = (config) => ({
  ...config,
  ky: api
});
