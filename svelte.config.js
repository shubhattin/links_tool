import adapter from '@sveltejs/adapter-vercel';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      runtime: 'edge' // as there is no node dependency now
    }),
    alias: {
      '@tools/*': 'src/tools/*',
      '@db/*': 'src/db/*',
      '@db': 'src/db/index.ts'
    }
  }
};

export default config;
