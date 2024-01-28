import { fetch_post, fetch_get, Fetch } from '@tools/fetch';
import { env } from '$env/dynamic/private';

let KEY: string = null!;
if (import.meta.env.DEV) KEY = env.DETA_PROJECT_KEY;
else KEY = process.env.DETA_PROJECT_KEY!;

const URL = (baseName: string) => `https://database.deta.sh/v1/${KEY?.split('_')[0]}/${baseName}`;

export const base_fetch = async (baseName: string, last: string = null!) => {
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
    items: any[];
  } = await resp.json();
  return json;
};

export const base_get = async (baseName: string, key: string) => {
  const req = fetch_get(`${URL(baseName)}/items/${key}`, {
    headers: {
      'X-Api-Key': KEY!
    }
  });
  const resp = await req;
  return await resp.json();
};

export const base_put = async (baseName: string, values: any[]) => {
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
  return await resp.json();
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
