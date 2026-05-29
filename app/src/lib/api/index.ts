/**
 * App-facing API layer: nested SDK on a single ky client.
 */
import { client as httpClient } from './generated/client.gen';
import { Api } from './generated/sdk.gen';
export * from './generated/@tanstack/svelte-query.gen';
export { client_q } from './query';

export const client = new Api({ client: httpClient, key: 'auth' });

// export { Api, Auth, Redirect } from './generated/sdk.gen';
export type * from './generated/types.gen';
export * from './generated/zod.gen';
