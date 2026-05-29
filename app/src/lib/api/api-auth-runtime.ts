/**
 * Ky runtime for authenticated API calls (401 → refresh → single retry).
 */
import type { CreateClientConfig } from './generated/client.gen';
import { authApi } from '@tools/ky';

export const createClientConfig: CreateClientConfig = (config) => ({
  ...config,
  ky: authApi
});
