import { HTTPError } from 'ky';
import {
  zErrorBody,
  zSessionResponse,
  zSignInBody,
  zSignUpBody,
  zUserDto,
  type SessionResponse,
  type SignInBody,
  type SignUpBody,
  type UserDto
} from '$lib/api/api_types';
import { api } from '$lib/api/ky';
import { queryClient } from './query_client';

export type AuthUser = UserDto;

/** Must match backend `ADMIN_ROLE` */
export const ADMIN_ROLE = 'admin';

export function isAdminUser(user: AuthUser | null | undefined): user is AuthUser {
  return user?.role === ADMIN_ROLE;
}
export type SignUpInput = SignUpBody;
export type SignInInput = SignInBody;
export type { SessionResponse };

export { zUserDto as authUserSchema, zSessionResponse as sessionResponseSchema };

export const AUTH_QUERY_KEY = ['auth', 'me'] as const;

/** Must match backend `ACCESS_TOKEN_TTL_SECS` */
const ACCESS_TOKEN_TTL_SECS = 15 * 60;

/** One retry after `/refresh` when the access cookie expired but refresh is still valid. */
const AUTH_ME_RETRY_LIMIT = 1;

type AuthSessionResult = { ok: true; data: SessionResponse } | { ok: false; error: string };

/** Thrown on 401 from `/me` so TanStack Query can retry once after token refresh. */
class AuthSessionExpiredError extends Error {
  constructor() {
    super('session expired');
    this.name = 'AuthSessionExpiredError';
  }
}

/** Non-auth failures loading the current user (network, 5xx, invalid payload). */
export class AuthFetchError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'AuthFetchError';
  }
}

let refreshTimer: ReturnType<typeof setTimeout> | null = null;
let tokenRefreshPromise: Promise<boolean> | null = null;

async function parseErrorMessage(error: unknown, fallback: string): Promise<string> {
  if (error instanceof HTTPError) {
    const body = await error.response
      .clone()
      .json()
      .catch(() => null);
    const parsed = zErrorBody.safeParse(body);
    if (parsed.success) return parsed.data.error;
  }
  return fallback;
}

async function fetchMeResponse(): Promise<AuthUser> {
  let raw: unknown;
  try {
    raw = await api.get('/api/auth/me').json();
  } catch (error) {
    if (error instanceof HTTPError && error.response.status === 401) {
      throw new AuthSessionExpiredError();
    }
    throw new AuthFetchError(await parseErrorMessage(error, 'failed to load session'));
  }

  const parsed = zUserDto.safeParse(raw);
  if (!parsed.success) {
    throw new AuthFetchError('invalid session response');
  }
  return parsed.data;
}

export async function fetchAuthUser(): Promise<AuthUser | null> {
  try {
    return await fetchMeResponse();
  } catch (error) {
    if (error instanceof AuthSessionExpiredError) {
      const refreshed = await refreshSessionTokens();
      if (!refreshed) return null;
      // TanStack Query retries once; the next call should succeed with the new access cookie.
      throw error;
    }
    throw error;
  }
}

export const authQueryOptions = () => ({
  queryKey: AUTH_QUERY_KEY,
  queryFn: fetchAuthUser,
  retry: (failureCount: number, error: Error) =>
    failureCount < AUTH_ME_RETRY_LIMIT && error instanceof AuthSessionExpiredError,
  retryDelay: 0
});

function scheduleRefresh(expiresInSec: number) {
  if (refreshTimer) clearTimeout(refreshTimer);
  const delay = Math.max(5_000, expiresInSec * 1000 * 0.8);
  refreshTimer = setTimeout(() => {
    void refreshSession();
  }, delay);
}

function applySessionExpiry(expiresIn: number) {
  scheduleRefresh(expiresIn);
}

export function clearSession() {
  if (refreshTimer) clearTimeout(refreshTimer);
  refreshTimer = null;
  queryClient.setQueryData(AUTH_QUERY_KEY, null);
}

async function refetchAuthUser(): Promise<AuthUser | null> {
  return queryClient.fetchQuery({
    queryKey: AUTH_QUERY_KEY,
    queryFn: fetchAuthUser,
    retry: authQueryOptions().retry,
    retryDelay: 0
  });
}

/** Refresh cookies only; does not refetch `/me`. */
export async function refreshSessionTokens(): Promise<boolean> {
  if (tokenRefreshPromise) return tokenRefreshPromise;

  tokenRefreshPromise = (async () => {
    try {
      const raw = await api.post('/api/auth/refresh').json();
      const parsed = zSessionResponse.safeParse(raw);
      if (!parsed.success) return false;

      applySessionExpiry(parsed.data.expires_in);
      return true;
    } catch {
      return false;
    }
  })().finally(() => {
    tokenRefreshPromise = null;
  });

  return tokenRefreshPromise;
}

export function refreshSession(): Promise<boolean> {
  return refreshSessionTokens().then(async (ok) => {
    if (!ok) {
      clearSession();
      return false;
    }
    try {
      const user = await refetchAuthUser();
      if (!user) {
        clearSession();
        return false;
      }
      return true;
    } catch {
      clearSession();
      return false;
    }
  });
}

/** After sign-in/sign-up: cookies are set; load profile from `/me`. */
export async function completeAuth(session: SessionResponse): Promise<boolean> {
  applySessionExpiry(session.expires_in);
  try {
    const user = await refetchAuthUser();
    if (!user) {
      clearSession();
      return false;
    }
    return true;
  } catch {
    clearSession();
    return false;
  }
}

export async function signOut() {
  try {
    await api.post('/api/auth/sign-out');
  } finally {
    clearSession();
  }
}

/** Existing cookie session on full page load (OAuth redirect, etc.). */
export function scheduleProactiveRefresh() {
  scheduleRefresh(ACCESS_TOKEN_TTL_SECS);
}

async function requestAuthSession(
  path: '/api/auth/sign-up' | '/api/auth/sign-in',
  body: unknown
): Promise<AuthSessionResult> {
  try {
    const raw = await api.post(path, { json: body }).json();
    const data = zSessionResponse.safeParse(raw);
    if (!data.success) {
      return { ok: false, error: 'invalid response' };
    }
    return { ok: true, data: data.data };
  } catch (err) {
    if (err instanceof HTTPError) {
      return { ok: false, error: await parseErrorMessage(err, 'request failed') };
    }
    return {
      ok: false,
      error: err instanceof Error && err.message ? err.message : 'network error'
    };
  }
}

export async function apiSignUp(body: SignUpInput): Promise<AuthSessionResult> {
  const input = zSignUpBody.safeParse(body);
  if (!input.success) {
    return { ok: false, error: input.error.issues[0]?.message ?? 'invalid input' };
  }
  return requestAuthSession('/api/auth/sign-up', input.data);
}

export async function apiSignIn(body: SignInInput): Promise<AuthSessionResult> {
  const input = zSignInBody.safeParse(body);
  if (!input.success) {
    return { ok: false, error: input.error.issues[0]?.message ?? 'invalid input' };
  }
  return requestAuthSession('/api/auth/sign-in', input.data);
}
