import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { proxy } from './proxy';

export default defineConfig({
  plugins: [sveltekit()],
  server: {
    proxy
  }
});