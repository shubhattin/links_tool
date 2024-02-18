import { redirect } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { db } from '@db';

export const GET: RequestHandler = async () => {
  return redirect(
    302,
    (await db.query.others.findFirst({
      where: ({ key }, { eq }) => eq(key, 'db_page')
    }))!.value
  );
};
