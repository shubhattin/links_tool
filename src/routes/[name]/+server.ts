import type { RequestHandler } from './$types';
import { z } from 'zod';
import { JSONResponse } from '@tools/responses';
import { get_redirect_response } from '$lib/server';
import { db } from '@db';

export const GET: RequestHandler = async ({ params }) => {
  const name_prsr = z.string().safeParse(params.name);
  if (!name_prsr.success) return JSONResponse({ detail: 'Wrong URL' });

  const name = name_prsr.data;
  const link_obj = await db.query.links.findFirst({
    where: ({ id }, { eq }) => eq(id, name)
  });
  if (!link_obj || !('link' in link_obj) || link_obj.link.includes('{0}'))
    return JSONResponse({ detail: 'Link Not Found' });
  return get_redirect_response(link_obj);
};
