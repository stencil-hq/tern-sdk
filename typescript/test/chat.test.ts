import { expect, test } from 'bun:test';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { editDraft } from '../examples/chat-draft.js';
import { KeyDecoder } from '../src/index.js';

test('table example renders an entry named head without colliding with its header', async () => {
   const directory = mkdtempSync(fileURLToPath(new URL('../.table-test-', import.meta.url)));
   try {
      writeFileSync(join(directory, 'head'), 'hello');
      // Load only after chdir: the example reads its directory at module initialization.
      const child = Bun.spawn(
         [
            process.execPath,
            '-e',
            `process.chdir(${JSON.stringify(directory)}); await import("./examples/table.tsx");`,
         ],
         {
            cwd: fileURLToPath(new URL('../', import.meta.url)),
            env: { ...process.env, TERN_TSP: '0' },
            stdout: 'pipe',
            stderr: 'pipe',
         }
      );
      const [stdout, stderr, code] = await Promise.all([
         new Response(child.stdout).text(),
         new Response(child.stderr).text(),
         child.exited,
      ]);
      expect(code).toBe(0);
      expect(stderr).toBe('');
      expect(stdout).toContain('Name');
      expect(stdout).toContain('head');
   } finally {
      rmSync(directory, { recursive: true, force: true });
   }
});

test('chat edits pasted emoji at code point boundaries with UTF-16 cursors', () => {
   const draft = { text: '', cursor: 0 };
   const keys = new KeyDecoder();
   const type = (input: string): void => {
      for (const key of keys.feed(new TextEncoder().encode(input))) editDraft(draft, key);
   };
   type('\x1b[200~a😀漢\x1b[201~');
   expect(draft).toEqual({ text: 'a😀漢', cursor: 4 });
   type('\x1b[D\x1b[D');
   expect(draft.cursor).toBe(1);
   type('\x1b[C');
   expect(draft.cursor).toBe(3);
   type('\x7f');
   expect(draft).toEqual({ text: 'a漢', cursor: 1 });
   type('😀\x1b[D\x1b[3~');
   expect(draft).toEqual({ text: 'a漢', cursor: 1 });
   type('\x1b[H\x1b[D');
   expect(draft.cursor).toBe(0);
   type('\x1b[F\x1b[C');
   expect(draft.cursor).toBe(2);
});
