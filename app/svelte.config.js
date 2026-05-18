import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/**
 * @type {import('@sveltejs/kit').Config}
 * Local dev: Rust on :5778, then `bun run dev`. Proxies: `vite.config.ts` (vercel.json parity).
 */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter(),
    alias: {
      '@tools/*': 'src/tools/*'
    }
  }
};

export default config;
