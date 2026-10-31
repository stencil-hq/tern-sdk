/** `print`: the current directory as a `table`, sizes right-aligned in a mono column. */
import { readdirSync, statSync } from 'node:fs';
import { print } from '@stencil-hq/tern';

/** A byte count as `312`, `2.1K`, `14M`. */
function human(bytes: number): string {
   const units = ['', 'K', 'M', 'G', 'T'];
   let value = bytes;
   let unit = 0;
   while (value >= 1024 && unit < units.length - 1) {
      value /= 1024;
      unit += 1;
   }
   const shown = unit === 0 || value >= 10 ? Math.round(value).toString() : value.toFixed(1);
   return shown + (units[unit] ?? '');
}

const entries = readdirSync('.', { withFileTypes: true })
   .filter(entry => !entry.name.startsWith('.'))
   .map(entry => ({
      name: entry.isDirectory() ? `${entry.name}/` : entry.name,
      dir: entry.isDirectory(),
      size: entry.isDirectory() ? '-' : human(statSync(entry.name).size),
   }))
   .sort((a, b) => Number(b.dir) - Number(a.dir) || a.name.localeCompare(b.name));

await print(
   <table
      cols={[
         { id: 'name', head: 'Name', mono: true, grow: 1 },
         { id: 'size', head: 'Size', align: 'end', mono: true },
      ]}
      rows={entries.map(entry => ({
         id: entry.name,
         cells: {
            name: { text: entry.name, icon: entry.dir ? 'folder' : 'file' },
            size: entry.size,
         },
      }))}
   />
);
