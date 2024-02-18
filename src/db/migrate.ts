import dotenv from 'dotenv';
import { migrate } from 'drizzle-orm/neon-http/migrator';
import { drizzle } from 'drizzle-orm/neon-http';
import { neon } from '@neondatabase/serverless';
import { get_db_url } from './db_utils';

dotenv.config({ path: '../../.env.local' });

export const migrationClient = neon(get_db_url());

// This will run migrations on the database, skipping the ones already applied
await migrate(drizzle(migrationClient), { migrationsFolder: './migrations' });
console.log('Migration Done.');

// await migrationClient();
