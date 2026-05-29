import { sessionResponseSchema, type SessionResponse } from '$lib/auth';
import { api } from '@tools/ky';
import { z } from 'zod';

export const signUpInputSchema = z.object({
  email: z.email().max(320),
  password: z.string().min(8).max(128),
  name: z.string().trim().min(1)
});

export const signInInputSchema = z.object({
  email: z.email().max(320),
  password: z.string().min(1).max(128)
});

const apiErrorSchema = z.object({
  error: z.string()
});

export type SignUpInput = z.infer<typeof signUpInputSchema>;
export type SignInInput = z.infer<typeof signInInputSchema>;

export async function apiSignUp(
  body: SignUpInput
): Promise<{ ok: true; data: SessionResponse } | { ok: false; error: string }> {
  const input = signUpInputSchema.safeParse(body);
  if (!input.success) {
    return { ok: false, error: input.error.issues[0]?.message ?? 'invalid input' };
  }

  try {
    const res = await api.post('/api/auth/sign-up', { json: input.data, throwHttpErrors: false });
    return parseSessionResponse(res);
  } catch (err) {
    return { ok: false, error: networkErrorMessage(err) };
  }
}

export async function apiSignIn(
  body: SignInInput
): Promise<{ ok: true; data: SessionResponse } | { ok: false; error: string }> {
  const input = signInInputSchema.safeParse(body);
  if (!input.success) {
    return { ok: false, error: input.error.issues[0]?.message ?? 'invalid input' };
  }

  try {
    const res = await api.post('/api/auth/sign-in', { json: input.data, throwHttpErrors: false });
    return parseSessionResponse(res);
  } catch (err) {
    return { ok: false, error: networkErrorMessage(err) };
  }
}

function networkErrorMessage(err: unknown): string {
  if (err instanceof Error && err.message) return err.message;
  return 'network error';
}

async function parseSessionResponse(
  res: Response
): Promise<{ ok: true; data: SessionResponse } | { ok: false; error: string }> {
  const payload = await res.json().catch(() => ({}));
  if (!res.ok) {
    const error = apiErrorSchema.safeParse(payload);
    return {
      ok: false,
      error: error.success ? error.data.error : 'request failed'
    };
  }

  const data = sessionResponseSchema.safeParse(payload);
  if (!data.success) {
    return { ok: false, error: 'invalid response' };
  }
  return { ok: true, data: data.data };
}
