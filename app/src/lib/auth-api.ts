import type { SessionResponse } from '$lib/auth';
import { fetch_post } from '@tools/fetch';

export type SignUpInput = {
  email: string;
  password: string;
  name: string;
};

export type SignInInput = {
  email: string;
  password: string;
};

export async function apiSignUp(
  body: SignUpInput
): Promise<{ ok: true; data: SessionResponse } | { ok: false; error: string }> {
  const res = await fetch_post('/api/auth/sign-up', { json: body, credentials: 'include' });
  return parseSessionResponse(res);
}

export async function apiSignIn(
  body: SignInInput
): Promise<{ ok: true; data: SessionResponse } | { ok: false; error: string }> {
  const res = await fetch_post('/api/auth/sign-in', { json: body, credentials: 'include' });
  return parseSessionResponse(res);
}

async function parseSessionResponse(
  res: Response
): Promise<{ ok: true; data: SessionResponse } | { ok: false; error: string }> {
  const payload = await res.json().catch(() => ({}));
  if (!res.ok) {
    const error = typeof payload?.error === 'string' ? payload.error : 'request failed';
    return { ok: false, error };
  }
  return { ok: true, data: payload as SessionResponse };
}
