import { writable } from 'svelte/store';
import { client, type SessionResponse, type UserDto, zSessionResponse, zUserDto } from '$lib/api';

export type AuthUser = UserDto;
export type { SessionResponse };

export { zUserDto as authUserSchema, zSessionResponse as sessionResponseSchema } from '$lib/api';

type AuthState = {
  user: AuthUser | null;
  expiresAt: number | null;
};

const initial: AuthState = {
  user: null,
  expiresAt: null
};

export const auth = writable<AuthState>(initial);

/** True while the initial `/api/auth/me` check (or explicit session bootstrap) is in flight. */
export const authLoading = writable(true);

let refreshTimer: ReturnType<typeof setTimeout> | null = null;

function scheduleRefresh(expiresInSec: number) {
  if (refreshTimer) clearTimeout(refreshTimer);
  const delay = Math.max(5_000, expiresInSec * 1000 * 0.8);
  refreshTimer = setTimeout(() => {
    void refreshSession();
  }, delay);
}

function applySessionExpiry(expiresIn: number) {
  const expiresAt = Date.now() + expiresIn * 1000;
  auth.update((s) => ({ ...s, expiresAt }));
  scheduleRefresh(expiresIn);
}

export function clearSession() {
  if (refreshTimer) clearTimeout(refreshTimer);
  refreshTimer = null;
  auth.set(initial);
}

/** Load the current user from the server */
export async function fetchMe(): Promise<{ ok: true; user: AuthUser } | { ok: false }> {
  try {
    const result = await client.auth.me({ throwOnError: false });
    if (result.error || !result.response?.ok) {
      return { ok: false };
    }
    const parsed = zUserDto.safeParse(result.data);
    if (!parsed.success) {
      return { ok: false };
    }
    auth.update((s) => ({ ...s, user: parsed.data }));
    return { ok: true, user: parsed.data };
  } catch {
    return { ok: false };
  }
}

let refreshPromise: Promise<boolean> | null = null;

async function runRefreshSession(): Promise<boolean> {
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
    const meResult = await fetchMe();
    if (!meResult.ok) {
      clearSession();
      return false;
    }
    return true;
  } catch {
    clearSession();
    return false;
  }
}

export function refreshSession(): Promise<boolean> {
  if (refreshPromise) return refreshPromise;

  refreshPromise = runRefreshSession().finally(() => {
    refreshPromise = null;
  });
  return refreshPromise;
}

/** After sign-in/sign-up: cookies are set; load profile from `/me`. */
export async function completeAuth(session: SessionResponse): Promise<boolean> {
  applySessionExpiry(session.expires_in);
  const meResult = await fetchMe();
  if (!meResult.ok) {
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

/** Must match backend `ACCESS_TOKEN_TTL_SECS` */
const ACCESS_TOKEN_TTL_SECS = 15 * 60;

export async function initAuth() {
  authLoading.set(true);
  try {
    const meResult = await fetchMe();
    if (!meResult.ok) {
      clearSession();
      return;
    }
    scheduleRefresh(ACCESS_TOKEN_TTL_SECS);
  } finally {
    authLoading.set(false);
  }
}

export function startAuthRefreshLoop() {
  void initAuth();
}
