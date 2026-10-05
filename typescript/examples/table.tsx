/** `print`: the current directory as a table, sizes right-aligned by a stylesheet. */
import { readdirSync, statSync } from 'node:fs';
import { css, html, print } from '@stencil-hq/tern';

const sheet = css`
   .ls {
      border-collapse: collapse;
   }
   .ls th,
   .ls td {
      padding: 1px 0 1px 24px;
   }
   .ls th:first-child,
   .ls td:first-child {
      padding-left: 0;
   }
   .ls th {
      text-align: left;
      font-weight: 600;
      color: var(--sf-c-muted);
   }
   .ls .size {
      text-align: right;
      font-variant-numeric: tabular-nums;
   }
   .ls .dir {
      color: var(--sf-c-accent);
   }
`;

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
   <html.table class="ls">
      <html.tr key="head">
         <html.th key="n">Name</html.th>
         <html.th key="s" class="size">
            Size
         </html.th>
      </html.tr>
      {entries.map(entry => (
         <html.tr key={`entry:${entry.name}`}>
            <html.td key="n" class={entry.dir ? 'dir' : 'file'}>
               {entry.name}
            </html.td>
            <html.td key="s" class="size">
               {entry.size}
            </html.td>
         </html.tr>
      ))}
   </html.table>,
   { css: sheet }
);
