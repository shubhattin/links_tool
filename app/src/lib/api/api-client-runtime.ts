/**
 * Runtime config for the generated Hey API ky client.
 * Uses plain `api` (no 401 retry) so sign-in/up/refresh/sign-out avoid refresh loops.
 * Pass `authApi` per call for cookie-protected routes (e.g. `me`).
 */
import type { CreateClientConfig } from './generated/client.gen';
import { api } from '@tools/ky';

export const createClientConfig: CreateClientConfig = (config) => ({
  ...config,
  ky: api
});
