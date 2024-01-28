import { fetch_post, fetch_get, Fetch } from '@tools/fetch';
import { env } from '$env/dynamic/private';

export interface key_value_type<T> {
  key: string;
  value: T;
}
const KEY = import.meta.env ? env.DETA_PROJECT_KEY : process.env.DETA_PROJECT_KEY!;

const URL = (baseName: string) => `https://database.deta.sh/v1/${KEY?.split('_')[0]}/${baseName}`;

export const base_fetch = async <T>(baseName: string, last: string = null!) => {
  const req = fetch_post(`${URL(baseName)}/query`, {
    json: {
      last: last
    },
    headers: {
      'X-Api-Key': KEY!
    }
  });
  const resp = await req;
  const json: {
    paging: {
      size: number;
      last?: string;
    };
    items: T[];
  } = await resp.json();
  return json;
};

export const base_fetch_all = async <T>(baseName: string) => {
  let last: string = null!;
  let list: T[] = [];
  while (true) {
    const dt = await base_fetch<T>(baseName, last);
    list = list.concat(dt.items);
    last = dt.paging.last!;
    if (!last) break;
  }
  return list;
};

export const base_get = async <T>(baseName: string, key: string) => {
  const req = fetch_get(`${URL(baseName)}/items/${key}`, {
    headers: {
      'X-Api-Key': KEY!
    }
  });
  const resp = await req;
  return (await resp.json()) as T;
};

export const base_put = async <T>(baseName: string, values: T[]) => {
  const req = Fetch(`${URL(baseName)}/items`, {
    method: 'PUT',
    json: {
      items: values
    },
    headers: {
      'X-Api-Key': KEY!
    }
  });
  const resp = await req;
};

export const base_delete = async (baseName: string, keys: string[]) => {
  const responses: Promise<Response>[] = [];
  for (let key of keys) {
    const req = Fetch(`${URL(baseName)}/items/${key}`, {
      method: 'DELETE',
      headers: {
        'X-Api-Key': KEY!
      }
    });
    responses.push(req);
  }
  for (let response of responses) await response;
};
