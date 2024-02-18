import { client } from './client';
import { readFile } from 'fs/promises';
import { take_input } from '@tools/kry';
import { links_table, others_table, selectLinksSchema, selectOthersSchema } from '@db/schema';
import { z } from 'zod';

const main = async () => {
  if (!(await confirm_environemnt())) return;

  console.log(`Insering Data into Database...`);

  const in_file_name = 'db_data.json';

  const data = z
    .object({
      others: selectOthersSchema.array(),
      links: selectLinksSchema.array()
    })
    .parse(JSON.parse((await readFile(`./out/${in_file_name}`)).toString()));

  await client.insert(links_table).values(data.links);
  console.log('Successfully added values into table `links`');
  await client.insert(others_table).values(data.others);
  console.log('Successfully added values into table `others`');
};
main();

async function confirm_environemnt() {
  let confirmation: string = await take_input(`Are you sure INSERT ? `);
  if (['yes', 'y'].includes(confirmation)) return true;
  return false;
}
