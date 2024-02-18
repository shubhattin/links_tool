import { pgTable, boolean, varchar, integer, text } from 'drizzle-orm/pg-core';
import { createSelectSchema } from 'drizzle-zod';

export const links_table = pgTable('links', {
  id: varchar('id', { length: 20 }).primaryKey(),
  enabled: boolean('enabled').default(true).notNull(),
  link: text('link').notNull(),
  prefix_zeros: integer('prefix_zeros').default(0).notNull(),
  name: varchar('name', { length: 30 })
});

export const others_table = pgTable('others', {
  key: varchar('key', { length: 20 }).primaryKey(),
  value: text('value').notNull()
});

export const schema = {
  others: others_table,
  links: links_table
};

export type Links = typeof links_table.$inferSelect;

export const selectLinksSchema = createSelectSchema(links_table);
export const selectOthersSchema = createSelectSchema(others_table);
