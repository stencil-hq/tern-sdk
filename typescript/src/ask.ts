/** The `ask` helper: one question in a form, its answer returned. */
import type { Renderable } from './reconcile.js';
import { connect, type ConnectOptions } from './session.js';
import type { FormValues } from './wire.js';

/** What `ask` takes. */
export interface AskOptions extends ConnectOptions {
   /** A stylesheet for the form. */
   readonly css?: string;
   /** The action that answers. Default `"submit"`. */
   readonly submit?: string;
}

/** The answer: the submitting node, its action and the form's values. */
export interface Answer {
   readonly type: 'answer';
   readonly id: string;
   readonly act: string;
   readonly values: FormValues;
}

/** Tern isn't there to ask: ask your own way. */
export interface Unsupported {
   readonly type: 'unsupported';
}

/**
 * Shows a form in a `flow` surface and returns the first `action` whose
 * `act` is `submit`, or `null` when the user presses Escape, Ctrl+C or
 * Ctrl+D. The form stays in the scrollback, and handlers on its nodes run
 * while it waits. Without TSP it returns `{type: 'unsupported'}`.
 */
export async function ask(
   view: Renderable,
   options: AskOptions = {}
): Promise<Answer | Unsupported | null> {
   const session = await connect(options);
   if (!session) return { type: 'unsupported' };
   const submit = options.submit ?? 'submit';
   try {
      const surface = session.open({ mode: 'flow', forwardActions: [submit] });
      if (options.css !== undefined) surface.stylesheet('main', options.css);
      surface.render(view);
      for await (const input of session) {
         if (input.type === 'key') {
            const { name, ctrl } = input.key;
            if (name === 'escape' || (ctrl && (name === 'c' || name === 'd'))) return null;
            continue;
         }
         const event = input.event;
         if (event.ev === 'action' && event.sf === surface.id && event.act === submit) {
            return { type: 'answer', id: event.id, act: event.act, values: event.values ?? {} };
         }
         if (event.ev === 'gone' && surface.closed) return null;
      }
      return null;
   } finally {
      await session.close();
   }
}
