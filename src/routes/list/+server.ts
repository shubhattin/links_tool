import { redirect } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { base_get } from '@tools/deta';

export const GET: RequestHandler = async () => {
  return redirect(302, (await base_get('others', 'db_page'))['value']);
};
