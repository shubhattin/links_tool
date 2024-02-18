import { env } from '$env/dynamic/private';
import { schema } from './schema';
import { drizzle } from 'drizzle-orm/neon-http';
import { neon } from '@neondatabase/serverless';
import { get_db_url } from './db_utils';

const DB_URL = get_db_url(env);

export const db = drizzle(neon(DB_URL), { schema });
