import dotenv from 'dotenv';
import { schema } from '@db/schema';
import { neon } from '@neondatabase/serverless';
import { drizzle } from 'drizzle-orm/neon-http';
import { get_db_url } from '@db/db_utils';

dotenv.config({ path: '../../../.env.local' });

export const queryClient = neon(get_db_url());
export const client = drizzle(queryClient, { schema });
