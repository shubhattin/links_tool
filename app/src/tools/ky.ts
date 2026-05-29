import ky, { type Options } from 'ky';

export const api = ky.create({
  credentials: 'include',
  hooks: {
    init: []
  }
});

/** Authenticated requests — retries once after refreshing the session on 401. */
export const authApi = api.extend({
  retry: { limit: 1 },
  hooks: {
    afterResponse: [
      async ({ response, retryCount }) => {
        if (response.status !== 401 || retryCount > 0) return;

        const { refreshSession } = await import('$lib/auth');
        const refreshed = await refreshSession();
        if (!refreshed) return;

        return ky.retry({ code: 'SESSION_REFRESHED' });
      }
    ]
  }
});
