import { client, queryClient } from './client';
import { writeFile } from 'fs/promises';
import { make_dir, take_input } from '@tools/kry';

const main = async () => {
  if (!(await confirm_environemnt())) return;

  console.log(`Fetching Data from Database...`);

  const links = await client.query.links.findMany();
  const others = await client.query.others.findMany();

  const json_data = {
    links,
    others
  };

  await make_dir('./out');
  const out_file_name = 'db_data.json';
  await writeFile(`./out/${out_file_name}`, JSON.stringify(json_data, null, 2));
};
main();

async function confirm_environemnt() {
  let confirmation: string = await take_input(`Are you sure SELECT ? `);
  if (['yes', 'y'].includes(confirmation)) return true;
  return false;
}
