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

type AuthSessionResult = { ok: true; data: SessionResponse } | { ok: false; error: string };

type SdkResult = {
  data?: unknown;
  error?: unknown;
  response?: Response;
};

let refreshTimer: ReturnType<typeof setTimeout> | null = null;
let refreshPromise: Promise<boolean> | null = null;

export async function fetchAuthUser(): Promise<AuthUser | null> {
  const result = await client.auth.me({ throwOnError: false });
  if (result.error || !result.response?.ok) return null;
  const parsed = zUserDto.safeParse(result.data);
  return parsed.success ? parsed.data : null;
}

export const authQueryOptions = () => ({
  queryKey: AUTH_QUERY_KEY,
  queryFn: fetchAuthUser,
  retry: false
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
    queryFn: fetchAuthUser
  });
}

export function refreshSession(): Promise<boolean> {
  if (refreshPromise) return refreshPromise;

  refreshPromise = (async () => {
    try {
      const result = await client.auth.refresh({ throwOnError: false });
      if (result.error || !result.response?.ok) {
        clearSession();
        return false;
      }
      const parsed = zSessionResponse.safeParse(result.data);
      if (!parsed.success) {
        clearSession();
        return false;
      }
      applySessionExpiry(parsed.data.expires_in);
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
  })().finally(() => {
    refreshPromise = null;
  });

  return refreshPromise;
}

/** After sign-in/sign-up: cookies are set; load profile from `/me`. */
export async function completeAuth(session: SessionResponse): Promise<boolean> {
  applySessionExpiry(session.expires_in);
  const user = await refetchAuthUser();
  if (!user) {
    clearSession();
    return false;
  }
  return true;
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
