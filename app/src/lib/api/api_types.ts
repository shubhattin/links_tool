import * as z from 'zod';

export const zSignUpBody = z.object({
  email: z.string(),
  name: z.string(),
  password: z.string()
});

export const zSignInBody = z.object({
  email: z.string(),
  password: z.string()
});

export const zUserDto = z.object({
  display_username: z.string().nullish(),
  email: z.string(),
  email_verified: z.boolean(),
  id: z.string(),
  image: z.string().nullish(),
  is_maintainer: z.boolean(),
  name: z.string(),
  role: z.string(),
  username: z.string().nullish()
});

export const zSessionResponse = z.object({
  expires_in: z.number().int()
});

export const zErrorBody = z.object({
  error: z.string()
});

export const zLinkDto = z.object({
  enabled: z.boolean(),
  id: z.string(),
  link: z.string(),
  name: z.string().nullish(),
  prefix_zeros: z.number().int()
});

export const zLinksListResponse = z.object({
  links: z.array(zLinkDto)
});

export const zCreateLinkBody = z.object({
  enabled: z.boolean(),
  id: z.string(),
  link: z.string(),
  name: z.string().nullish(),
  prefix_zeros: z.number().int()
});

export const zUpdateLinkBody = z.object({
  enabled: z.boolean(),
  link: z.string(),
  name: z.string().nullish(),
  prefix_zeros: z.number().int()
});

export const zDetailResponse = z.object({
  detail: z.string()
});

export type SignUpBody = z.infer<typeof zSignUpBody>;
export type SignInBody = z.infer<typeof zSignInBody>;
export type UserDto = z.infer<typeof zUserDto>;
export type SessionResponse = z.infer<typeof zSessionResponse>;
export type ErrorBody = z.infer<typeof zErrorBody>;
export type LinkDto = z.infer<typeof zLinkDto>;
export type LinksListResponse = z.infer<typeof zLinksListResponse>;
export type CreateLinkBody = z.infer<typeof zCreateLinkBody>;
export type UpdateLinkBody = z.infer<typeof zUpdateLinkBody>;
export type DetailResponse = z.infer<typeof zDetailResponse>;
