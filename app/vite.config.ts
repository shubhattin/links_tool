import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { proxy } from './proxy';

export default defineConfig(({ command }) => ({
  plugins: [sveltekit()],
  ...(command === 'serve'
    ? {
        server: {
          proxy
          // proxy only during development
        }
      }
    : {})
}));
