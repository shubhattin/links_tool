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

type AuthSessionResult = { ok: true; data: SessionResponse } | { ok: false; error: string };

type SdkResult = {
  data?: unknown;
  error?: unknown;
  response?: Response;
};

/** Shared sign-in/sign-up: parse SDK result, map API errors, handle network failures. */
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
