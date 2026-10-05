/** The `print` helper: a static view kept in the scrollback, like command output. */
import { plain } from './plain.js';
import type { Renderable } from './reconcile.js';
import { connect, type ConnectOptions } from './session.js';

/** What `print` takes. */
export interface PrintOptions extends ConnectOptions {
   /** A stylesheet for the view. */
   readonly css?: string;
   /** The text written outside Tern instead of the plain rendering. */
   readonly fallback?: string;
}

/**
 * Shows a static view that stays in the scrollback: a `flow` surface opened
 * with `listen: false`, its sheet, one frame, closed with `keep`. Outside
 * Tern (or when stdout is not a tty) it writes the plain-text rendering, or
 * `fallback`.
 */
export async function print(view: Renderable, options: PrintOptions = {}): Promise<void> {
   const session = await connect({ bracketedPaste: false, kittyKeyboard: false, ...options });
   if (!session) {
      const output = options.output ?? process.stdout;
      const text = options.fallback ?? plain(view, output.columns ?? 80);
      output.write(text.endsWith('\n') || text === '' ? text : `${text}\n`);
      return;
   }
   try {
      const surface = session.open({ mode: 'flow', listen: false });
      if (options.css !== undefined) surface.stylesheet('main', options.css);
      surface.render(view);
      await surface.close({ keep: true });
   } finally {
      await session.close();
   }
}
