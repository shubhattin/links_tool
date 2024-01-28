import { redirect } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { base_get } from '@tools/deta';
import { z } from 'zod';
import { JSONResponse } from '@tools/responses';
import { type link_obj_response_type, get_redirect_response } from '$lib/server';

export const GET: RequestHandler = async ({ params }) => {
  const name_prsr = z.string().safeParse(params.name);
  if (!name_prsr.success) return JSONResponse({ detail: 'Wrong URL' });

  const name = name_prsr.data;
  const link_obj = (await base_get('links', name)) as link_obj_response_type;
  if (!link_obj || !('link' in link_obj)) return JSONResponse({ detail: 'Link Not Found' });
  return get_redirect_response(link_obj);
};
