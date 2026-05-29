import { writable } from 'svelte/store';
import { z } from 'zod';
import { api, authApi } from '@tools/ky';

export const authUserSchema = z.object({
  id: z.string(),
  email: z.string(),
  name: z.string(),
  role: z.string(),
  email_verified: z.boolean()
});

export const sessionResponseSchema = z.object({
  expires_in: z.number()
});

export type AuthUser = z.infer<typeof authUserSchema>;
export type SessionResponse = z.infer<typeof sessionResponseSchema>;

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
    const res = await authApi.get('/api/auth/me', { throwHttpErrors: false });
    if (!res.ok) {
      return { ok: false };
    }
    const parsed = authUserSchema.safeParse(await res.json());
    if (!parsed.success) {
      return { ok: false };
    }
    auth.update((s) => ({ ...s, user: parsed.data }));
    return { ok: true, user: parsed.data };
  } catch {
    return { ok: false };
  }
}

export async function refreshSession(): Promise<boolean> {
  try {
    const res = await api.post('/api/auth/refresh', { throwHttpErrors: false });
    if (!res.ok) {
      clearSession();
      return false;
    }
    const parsed = sessionResponseSchema.safeParse(await res.json());
    if (!parsed.success) {
      clearSession();
      return false;
    }
    applySessionExpiry(parsed.data.expires_in);
    const me = await fetchMe();
    if (!me.ok) {
      clearSession();
      return false;
    }
    return true;
  } catch {
    clearSession();
    return false;
  }
}

/** After sign-in/sign-up: cookies are set; load profile from `/me`. */
export async function completeAuth(session: SessionResponse): Promise<boolean> {
  applySessionExpiry(session.expires_in);
  const me = await fetchMe();
  if (!me.ok) {
    clearSession();
    return false;
  }
  return true;
}

export async function signOut() {
  try {
    await api.post('/api/auth/sign-out', { throwHttpErrors: false });
  } finally {
    clearSession();
  }
}

/** Must match backend `ACCESS_TOKEN_TTL_SECS` */
const ACCESS_TOKEN_TTL_SECS = 15 * 60;

export async function initAuth() {
  authLoading.set(true);
  try {
    const me = await fetchMe();
    if (!me.ok) {
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
