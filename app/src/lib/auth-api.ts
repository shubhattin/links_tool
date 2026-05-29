import {
  client,
  type SessionResponse,
  type SignInBody,
  type SignUpBody,
  zErrorBody,
  zSessionResponse,
  zSignInBody,
  zSignUpBody
} from '$lib/api';

export type SignUpInput = SignUpBody;
export type SignInInput = SignInBody;
export type { SessionResponse };

export async function apiSignUp(
  body: SignUpInput
): Promise<{ ok: true; data: SessionResponse } | { ok: false; error: string }> {
  const input = zSignUpBody.safeParse(body);
  if (!input.success) {
    return { ok: false, error: input.error.issues[0]?.message ?? 'invalid input' };
  }

  try {
    const result = await client.auth.signUp({ body: input.data, throwOnError: false });
    return parseSessionResult(result);
  } catch (err) {
    return { ok: false, error: networkErrorMessage(err) };
  }
}

export async function apiSignIn(
  body: SignInInput
): Promise<{ ok: true; data: SessionResponse } | { ok: false; error: string }> {
  const input = zSignInBody.safeParse(body);
  if (!input.success) {
    return { ok: false, error: input.error.issues[0]?.message ?? 'invalid input' };
  }

  try {
    const result = await client.auth.signIn({ body: input.data, throwOnError: false });
    return parseSessionResult(result);
  } catch (err) {
    return { ok: false, error: networkErrorMessage(err) };
  }
}

function networkErrorMessage(err: unknown): string {
  if (err instanceof Error && err.message) return err.message;
  return 'network error';
}

type SdkResult = {
  data?: unknown;
  error?: unknown;
  response?: Response;
};

function parseSessionResult(
  result: SdkResult
): Promise<{ ok: true; data: SessionResponse } | { ok: false; error: string }> {
  if (result.error || !result.response?.ok) {
    const error = zErrorBody.safeParse(result.error);
    return Promise.resolve({
      ok: false,
      error: error.success ? error.data.error : 'request failed'
    });
  }

  const data = zSessionResponse.safeParse(result.data);
  if (!data.success) {
    return Promise.resolve({ ok: false, error: 'invalid response' });
  }
  return Promise.resolve({ ok: true, data: data.data });
}
