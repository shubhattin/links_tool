import readline from 'readline';
import { existsSync } from 'fs';
import { mkdir } from 'fs/promises';

export async function take_input(prompt: string) {
  return new Promise<string>((resolve) => {
    const rl = readline.createInterface({
      input: process.stdin,
      output: process.stdout
    });
    rl.question(prompt, (answer) => {
      rl.close();
      resolve(answer);
    });
  });
}

export async function make_dir(folderPath: string) {
  if (!existsSync(folderPath)) await mkdir(folderPath, { recursive: true });
}
