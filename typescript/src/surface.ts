/**
 * A surface: a document the program shows in its pane, rendered from views
 * under credit-based flow control, with view ops, stylesheets, a palette and
 * event routing to node handlers.
 */
import { View, type Renderable } from './reconcile.js';
import type {
   OutMessage,
   Op,
   PaletteColors,
   RevealAt,
   ScrollTo,
   SurfaceMode,
   TspEvent,
} from './wire.js';

/** What `open` takes. */
export interface OpenOptions {
   /** The surface id; `s1`, `s2`, … per session by default. */
   readonly id?: string;
   /** Default `inline`. */
   readonly mode?: SurfaceMode;
   /** Names the pane until the program sets its own title. */
   readonly title?: string;
   /** `data-surface` on the surface's regions, for stylesheets. */
   readonly role?: string;
   /** `false`: the program never reads input; no acks, frames go at once. Default `true`. */
   readonly listen?: boolean;
   /** Reopen the closed inline surface with this id. */
   readonly adopt?: boolean;
   /** Also yield these action names after their node handlers run (used by `ask`). */
   readonly forwardActions?: readonly string[];
}

/** What `close` takes. */
export interface CloseOptions {
   /** Leave the surface's `main` in the scrollback. Default `true`. */
   readonly keep?: boolean;
}

/** The program palette: one token map per appearance and their names. */
export interface Palette {
   readonly dark?: PaletteColors;
   readonly light?: PaletteColors;
   readonly name?: { readonly dark?: string; readonly light?: string };
}

/** What a surface needs from its session. */
export interface SurfaceLink {
   /** Writes a message (and records it). */
   send(message: OutMessage): void;
   /** The frames that may be unacknowledged. */
   credits(): number;
   /** The surface closed or went away. */
   dropped(surface: Surface): void;
}

/** A pending handler call for one event. */
export type Dispatch = () => void | Promise<void>;

/**
 * An open surface. `render` sends only the difference from the last view
 * sent, at most `credits` frames ahead of Tern's acks; while blocked it
 * keeps the newest view and the queued ops for one frame when credit returns.
 */
export class Surface implements AsyncDisposable {
   /** The surface id (`sf` on frames and events). */
   readonly id: string;
   readonly mode: SurfaceMode;
   /** Whether Tern sends acks and events for it. */
   readonly listen: boolean;
   #link: SurfaceLink;
   /** The last frame number sent. */
   #seq = 0;
   /** The highest frame number acked. */
   #acked = 0;
   /** The view last sent. */
   #sent = new View();
   /** A view rendered while blocked. */
   #pending: View | undefined;
   /** The view last rendered: events route to its nodes. */
   #latest = new View();
   /** View and raw ops waiting for credit. */
   #queue: Op[] = [];
   #closed = false;
   #resetMain = false;
   readonly forwardActions: readonly string[];

   constructor(link: SurfaceLink, id: string, options: OpenOptions, previous?: Surface) {
      this.#link = link;
      this.id = id;
      this.mode = options.mode ?? 'inline';
      this.listen = options.listen ?? true;
      this.forwardActions = options.forwardActions ?? [];
      if (options.adopt) {
         if (previous) {
            this.#sent = previous.#sent;
            this.#latest = this.#sent;
            this.#seq = previous.#seq;
            this.#acked = this.#seq;
         } else this.#resetMain = true;
      }
   }

   /** Whether the surface is closed (by `close`, or a `gone` from Tern). */
   get closed(): boolean {
      return this.#closed;
   }

   /** Whether frames wait for an ack. */
   get blocked(): boolean {
      return this.listen && this.#seq - this.#acked >= this.#link.credits();
   }

   /** The last frame number sent. */
   get seq(): number {
      return this.#seq;
   }

   /** Shows `view`: a view by region, or nodes for `main`. Throws a `ViewError` for a bad view. */
   render(view: Renderable): void {
      if (this.#closed) return;
      const next = View.from(view);
      this.#latest = next;
      this.#pending = next;
      this.#pump();
   }

   /** Installs (or with `null` removes) stylesheet `name`. */
   stylesheet(name: string, css: string | null): void {
      if (this.#closed) return;
      this.#link.send({
         verb: 's',
         body: css === null || css === '' ? { sf: this.id, name } : { sf: this.id, name, css },
      });
   }

   /** Sends the program palette. */
   palette(palette: Palette): void {
      if (this.#closed) return;
      this.#link.send({ verb: 't', body: { sf: this.id, ...palette } });
   }

   /** Gives the caret to an `editor` or `input`, or to none. */
   focus(id: string | null): void {
      this.send([['focus', id]]);
   }

   /** Scrolls a node into view. */
   reveal(id: string, at: RevealAt = 'nearest'): void {
      this.send([['reveal', id, at]]);
   }

   /** Scrolls the scroll container at or above a node. */
   scroll(id: string, to: ScrollTo): void {
      this.send([['scroll', id, to]]);
   }

   /** Hints that a subtree is unlikely to change soon. */
   settle(id: string): void {
      this.send([['settle', id]]);
   }

   /** Hands the pane back to the grid. */
   suspend(): void {
      this.send([['suspend']]);
   }

   /** Takes the pane again after `suspend`. */
   resume(): void {
      this.send([['resume']]);
   }

   /** Queues raw ops for the next frame (after the view's difference). */
   send(ops: readonly Op[]): void {
      if (this.#closed) return;
      this.#queue.push(...ops);
      this.#pump();
   }

   /**
    * Closes the surface: what is still pending goes as a last frame (credit
    * or not, so the kept surface shows the final view), then `x`.
    */
   close(options: CloseOptions = {}): Promise<void> {
      if (this.#closed) return Promise.resolve();
      let failure: { error: unknown } | undefined;
      try {
         this.#flush();
      } catch (error) {
         failure = { error };
      }
      try {
         const keep = options.keep ?? true;
         this.#link.send({ verb: 'x', body: { id: this.id, keep } });
         this.#sent = keep ? new View([this.#sent.regions[0], undefined, undefined]) : new View();
         this.#closed = true;
         this.#link.dropped(this);
      } catch (error) {
         failure ??= { error };
      }
      if (failure) throw failure.error;
      return Promise.resolve();
   }

   /** Closes with `keep`, for `await using`. */
   [Symbol.asyncDispose](): Promise<void> {
      return this.close();
   }

   /** Takes an `ack` for frame `s`: credit for it and every frame before. */
   acknowledge(s: number): void {
      if (s <= this.#acked) return;
      this.#acked = Math.min(s, this.#seq);
      this.#pump();
   }

   /** Tern dropped the surface: it takes no more frames. */
   gone(): void {
      if (this.#closed) return;
      this.#closed = true;
      this.#link.dropped(this);
   }

   /** The handler call of a node in the view last rendered for a user event, if it has one. */
   dispatch(event: TspEvent): Dispatch | undefined {
      if (!('id' in event) || event.id === undefined) return undefined;
      const handlers = this.#latest.find(event.id)?.node.handlers;
      if (!handlers) return undefined;
      switch (event.ev) {
         case 'action': {
            const actions = this.#latest.find(event.id)?.props['actions'];
            const full = event.value === undefined ? event.act : `${event.act}=${event.value}`;
            const named = (gesture: string): boolean =>
               typeof actions === 'object' &&
               actions !== null &&
               !Array.isArray(actions) &&
               Reflect.get(actions, gesture) === full;
            const fn =
               handlers.onAction?.[full] ??
               handlers.onAction?.[event.act] ??
               handlers.onMenu?.[full] ??
               handlers.onMenu?.[event.act] ??
               (named('click') ? handlers.onClick : undefined) ??
               (named('dblclick') ? handlers.onDblClick : undefined);
            return fn && (() => fn(event));
         }
         case 'toggle': {
            const fn = handlers.onToggle;
            return fn && (() => fn(event));
         }
         case 'select': {
            const fn = handlers.onSelect;
            return fn && (() => fn(event));
         }
         case 'activate': {
            const fn = handlers.onActivate;
            return fn && (() => fn(event));
         }
         case 'change': {
            const fn = handlers.onChange;
            return fn && (() => fn(event));
         }
         case 'focus': {
            const fn = handlers.onFocus;
            return fn && (() => fn(event));
         }
         case 'edit': {
            const fn = handlers.onEdit;
            return fn && (() => fn(event));
         }
         case 'undo': {
            const fn = handlers.onUndo;
            return fn && (() => fn(event));
         }
         case 'send': {
            const fn = handlers.onSend;
            return fn && (() => fn(event));
         }
         default:
            return undefined;
      }
   }

   /** Sends one frame when credit allows and there is anything to send. */
   #pump(): void {
      if (this.#closed || this.blocked) return;
      this.#flush();
   }

   /** Sends the pending difference and queued ops as one frame, when there are any. */
   #flush(): void {
      const ops: Op[] = [];
      const next = this.#pending;
      if (this.#resetMain && (next || this.#queue.length > 0)) ops.push(['del', 'main']);
      if (next) ops.push(...this.#sent.ops(next, this.id));
      ops.push(...this.#queue);
      if (ops.length > 0) {
         const seq = this.#seq + 1;
         this.#link.send({ verb: 'f', body: { sf: this.id, s: seq, ops } });
         this.#seq = seq;
         this.#resetMain = false;
      }
      this.#queue = [];
      if (next) {
         this.#sent = next;
         this.#pending = undefined;
      }
   }
}
