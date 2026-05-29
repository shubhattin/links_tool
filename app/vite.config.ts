import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { proxy } from './proxy';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig(({ command }) => ({
  plugins: [tailwindcss(), sveltekit()],
  ...(command === 'serve'
    ? {
        server: {
          proxy
          // proxy only during development
        }
      }
    : {})
}));
