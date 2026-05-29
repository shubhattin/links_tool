import {
  authMeOptions,
  authMeQueryKey,
  authRefreshMutation,
  authSignInMutation,
  authSignOutMutation,
  authSignUpMutation,
  redirectByNameNumOptions,
  redirectByNameNumQueryKey,
  redirectByNameOptions,
  redirectByNameQueryKey
} from './generated/@tanstack/svelte-query.gen';

// This needs to manually updated by after change in axum backend and type generation
export const client_q = {
  auth: {
    me: {
      queryOptions: authMeOptions,
      queryKey: authMeQueryKey
    },
    refresh: {
      mutationOptions: authRefreshMutation
    },
    signIn: {
      mutationOptions: authSignInMutation
    },
    signOut: {
      mutationOptions: authSignOutMutation
    },
    signUp: {
      mutationOptions: authSignUpMutation
    }
  },
  redirect: {
    byName: {
      queryOptions: redirectByNameOptions,
      queryKey: redirectByNameQueryKey
    },
    byNameNum: {
      queryOptions: redirectByNameNumOptions,
      queryKey: redirectByNameNumQueryKey
    }
  }
};
