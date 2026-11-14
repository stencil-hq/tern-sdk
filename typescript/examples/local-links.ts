/** Compare native Markdown and span links without an agent or a Markdown rewriter. */
import { statSync } from 'node:fs';
import { relative, resolve, sep } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { connect, span, ui } from '@stencil-hq/tern';

const file = resolve(process.argv[2] ?? fileURLToPath(import.meta.url));
if (!statSync(file).isFile()) throw new Error(`Expected a local file: ${file}`);
const fileUrl = pathToFileURL(file);
const absoluteTarget = fileUrl.pathname;
const relativeTarget = relative(process.cwd(), file).split(sep).map(encodeURIComponent).join('/');

await using session = await connect({ app: 'local-links' });
if (!session) throw new Error('Run this example in a Tern terminal pane with TSP enabled.');
const surface = session.open({ mode: 'flow', listen: false });
surface.render(
   ui.col(
      ui.text(`Local file: ${file}`),
      ui.md(`[Markdown file URL](<${fileUrl.href}>)`),
      ui.md(`[Markdown absolute path](<${absoluteTarget}>)`),
      ui.md(`[Markdown relative path](<${relativeTarget}>)`),
      ui.md('[Markdown HTTPS](https://example.com/)'),
      ui.text(span('Native span file URL', undefined, { href: fileUrl.href })),
      ui.text(span('Native span absolute path', undefined, { href: absoluteTarget }))
   )
);
await surface.close({ keep: true });
