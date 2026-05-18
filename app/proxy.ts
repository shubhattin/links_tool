import type { ProxyOptions } from "vite";

const AXUM_DEV_PORT = 5778;
const AXUM_DEV_TARGET =
    process.env.AXUM_DEV_ORIGIN ?? `http://127.0.0.1:${AXUM_DEV_PORT}`;

// Mirror `../vercel.json` rewrites during local dev:
// - /api/:path*
// - /:name
// - /:name/:num
//
// We explicitly exclude Vite/SvelteKit internals and static file-like paths.
//
// SINGLE_SEGMENT_REDIRECT_PATH:
// - Matches one path segment like `/:name` (example: `/google`, `/yt`).
// - Does NOT match reserved/internal prefixes (`/api`, `/src`, `/_app`, `/@...`).
// - Does NOT match file-like paths with dots (example: `/logo.png`).
const SINGLE_SEGMENT_REDIRECT_PATH =
    '^/(?!api(?:/|$)|@|src(?:/|$)|node_modules(?:/|$)|_app(?:/|$)|__vite_ping$)[^/.@][^/]*$';
//
// TWO_SEGMENT_REDIRECT_PATH:
// - Matches two path segments like `/:name/:num` (example: `/google/1.25`).
// - Same exclusions for Vite/SvelteKit internals (`/api`, `/src`, `/_app`, `/@...`).
// - Prevents obvious static file paths in the second segment (example: `/x/file.js`).
const TWO_SEGMENT_REDIRECT_PATH =
    '^/(?!api(?:/|$)|@|src(?:/|$)|node_modules(?:/|$)|_app(?:/|$))[^/.@][^/]*/[^/.][^/]*$';

const AXUM_PROXY_PATTERNS = [
    '^/api(?:/.*)?$',
    SINGLE_SEGMENT_REDIRECT_PATH,
    TWO_SEGMENT_REDIRECT_PATH
] as const;

const axumProxy: ProxyOptions = {
    target: AXUM_DEV_TARGET,
    changeOrigin: true,
    secure: false
};

export const proxy = Object.fromEntries(
    AXUM_PROXY_PATTERNS.map((pattern) => [pattern, axumProxy])
) as Record<string, ProxyOptions>;
