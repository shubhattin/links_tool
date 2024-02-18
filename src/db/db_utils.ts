import { z } from 'zod';

export const get_db_url = (env: any = process.env) => {
  const url_parse = z
    .string({
      description: 'Connection string for PostgreSQL'
    })
    .safeParse(env.PG_DATABASE_URL);
  if (!url_parse.success) throw new Error('Please set `PG_DATABASE_URL` in .env.local');
  return url_parse.data;
};
