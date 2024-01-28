import { redirect } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { base_get, type key_value_type } from '@tools/deta';

export const GET: RequestHandler = async () => {
  return redirect(302, (await base_get<key_value_type<string>>('others', 'db_page')).value);
};
