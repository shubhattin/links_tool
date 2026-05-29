import { writable } from 'svelte/store';

export type AuthUser = {
  id: string;
  email: string;
  name: string;
  role: string;
  email_verified: boolean;
};

/** Cookie session metadata only — no JWT in the client. */
export type SessionResponse = {
  expires_in: number;
};

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

/** Load the current user from the server (never decode JWT client-side). */
export async function fetchMe(): Promise<{ ok: true; user: AuthUser } | { ok: false }> {
  try {
    const res = await fetch('/api/auth/me', { credentials: 'include' });
    if (!res.ok) {
      return { ok: false };
    }
    const user = (await res.json()) as AuthUser;
    auth.update((s) => ({ ...s, user }));
    return { ok: true, user };
  } catch {
    return { ok: false };
  }
}

export async function refreshSession(): Promise<boolean> {
  try {
    const res = await fetch('/api/auth/refresh', {
      method: 'POST',
      credentials: 'include'
    });
    if (!res.ok) {
      clearSession();
      return false;
    }
    const data = (await res.json()) as SessionResponse;
    applySessionExpiry(data.expires_in);
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
    await fetch('/api/auth/sign-out', { method: 'POST', credentials: 'include' });
  } finally {
    clearSession();
  }
}

/** Must match backend `ACCESS_TOKEN_TTL_SECS` (scheduling only, not JWT parsing). */
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
