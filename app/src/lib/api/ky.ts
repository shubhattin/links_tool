import ky from 'ky';

/** Routes that must not trigger 401 → refresh → retry (avoids refresh loops). */
const NO_SESSION_REFRESH_PATHS = [
  '/api/auth/refresh',
  '/api/auth/sign-in',
  '/api/auth/sign-up',
  '/api/auth/sign-out'
] as const;

export const api = ky.create({
  credentials: 'include',
  retry: { limit: 1 },
  hooks: {
    afterResponse: [
      async ({ request, response, retryCount }) => {
        if (response.status !== 401 || retryCount > 0) return;

        // Skip refresh retry on auth endpoints (avoids loops).
        const pathname = new URL(request.url).pathname;
        if (NO_SESSION_REFRESH_PATHS.some((path) => pathname.endsWith(path))) return;

        const { refreshSession } = await import('$lib/auth.svelte');
        const refreshed = await refreshSession();
        if (!refreshed) return;

        return ky.retry({ code: 'SESSION_REFRESHED' });
      }
    ]
  }
});
