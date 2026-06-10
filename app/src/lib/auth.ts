import {
  client,
  type SessionResponse,
  type SignInBody,
  type SignUpBody,
  type UserDto,
  zErrorBody,
  zSessionResponse,
  zSignInBody,
  zSignUpBody,
  zUserDto
} from '$lib/api';
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

export { zUserDto as authUserSchema, zSessionResponse as sessionResponseSchema } from '$lib/api';

export const AUTH_QUERY_KEY = ['auth', 'me'] as const;

/** Must match backend `ACCESS_TOKEN_TTL_SECS` */
const ACCESS_TOKEN_TTL_SECS = 15 * 60;

/** One retry after `/refresh` when the access cookie expired but refresh is still valid. */
const AUTH_ME_RETRY_LIMIT = 1;

type AuthSessionResult = { ok: true; data: SessionResponse } | { ok: false; error: string };

type SdkResult = {
  data?: unknown;
  error?: unknown;
  response?: Response;
};

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

async function fetchMeResponse(): Promise<AuthUser> {
  const result = await client.auth.me({ throwOnError: false });

  if (result.response?.status === 401) {
    throw new AuthSessionExpiredError();
  }
  if (result.error || !result.response?.ok) {
    throw new AuthFetchError('failed to load session');
  }

  const parsed = zUserDto.safeParse(result.data);
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
      const result = await client.auth.refresh({ throwOnError: false });
      if (result.error || !result.response?.ok) return false;

      const parsed = zSessionResponse.safeParse(result.data);
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
    await client.auth.signOut({ throwOnError: false });
  } finally {
    clearSession();
  }
}

/** Existing cookie session on full page load (OAuth redirect, etc.). */
export function scheduleProactiveRefresh() {
  scheduleRefresh(ACCESS_TOKEN_TTL_SECS);
}

async function requestAuthSession(call: () => Promise<SdkResult>): Promise<AuthSessionResult> {
  try {
    const result = await call();
    if (result.error || !result.response?.ok) {
      const error = zErrorBody.safeParse(result.error);
      return {
        ok: false,
        error: error.success ? error.data.error : 'request failed'
      };
    }
    const data = zSessionResponse.safeParse(result.data);
    if (!data.success) {
      return { ok: false, error: 'invalid response' };
    }
    return { ok: true, data: data.data };
  } catch (err) {
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
  return requestAuthSession(() => client.auth.signUp({ body: input.data, throwOnError: false }));
}

export async function apiSignIn(body: SignInInput): Promise<AuthSessionResult> {
  const input = zSignInBody.safeParse(body);
  if (!input.success) {
    return { ok: false, error: input.error.issues[0]?.message ?? 'invalid input' };
  }
  return requestAuthSession(() => client.auth.signIn({ body: input.data, throwOnError: false }));
}
