import ky from 'ky';

/** Routes that must not trigger 401 → refresh → retry (avoids refresh loops). */
const NO_SESSION_REFRESH_PATHS = [
  '/api/auth/refresh',
  '/api/auth/sign-in',
  '/api/auth/sign-up',
  '/api/auth/sign-out'
] as const;

function shouldRefreshSessionOn401(pathname: string): boolean {
  return !NO_SESSION_REFRESH_PATHS.some((path) => pathname.endsWith(path));
}

export const api = ky.create({
  credentials: 'include',
  retry: { limit: 1 },
  hooks: {
    afterResponse: [
      async ({ request, response, retryCount }) => {
        if (response.status !== 401 || retryCount > 0) return;

        const pathname = new URL(request.url).pathname;
        if (!shouldRefreshSessionOn401(pathname)) return;

        const { refreshSession } = await import('$lib/auth');
        const refreshed = await refreshSession();
        if (!refreshed) return;

        return ky.retry({ code: 'SESSION_REFRESHED' });
      }
    ]
  }
});
