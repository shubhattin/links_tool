import { refreshSession } from '$lib/auth';
import { AharaNam, type options } from '@tools/fetch';

/** Authenticated fetch — relies on httpOnly cookies, not Authorization headers. */
export async function fetchAuth(url: string, op: options = {}) {
  op.credentials = 'include';

  let res = await AharaNam(url, { ...op });
  if (res.status !== 401) return res;

  const refreshed = await refreshSession();
  if (!refreshed) return res;

  return AharaNam(url, { ...op });
}

export const fetch_auth_get = (url: string, op: options = {}) => {
  op.method = 'GET';
  return fetchAuth(url, op);
};

export const fetch_auth_post = (url: string, op: options = {}) => {
  op.method = 'POST';
  return fetchAuth(url, op);
};
