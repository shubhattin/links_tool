import { redirect } from '@sveltejs/kit';
import { JSONResponse } from '@tools/responses';

export interface link_obj_response_type {
  key: string;
  enabled: boolean;
  link: string;
  prefix_zeros: number;
}

const relU = (num: number) => (num >= 0 ? num : 0);

export const get_redirect_response = (link_obj: link_obj_response_type, num: number = 0) => {
  let { link } = link_obj;
  const { enabled, prefix_zeros } = link_obj;
  if (!enabled) return JSONResponse({ detail: 'Link Disabled' });
  link = link.replace(
    '{0}',
    '0'.repeat(relU(prefix_zeros - num.toString().length)) + num.toString()
  );
  return redirect(302, link);
};
