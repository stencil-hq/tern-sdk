import type { Key } from '@stencil-hq/tern';

/** The editor's text and UTF-16 caret offset. */
export interface Draft {
   text: string;
   cursor: number;
}

/** Move a UTF-16 caret by one code point, without splitting a surrogate pair. */
function step(text: string, cursor: number, direction: -1 | 1): number {
   if (direction === 1)
      return Math.min(text.length, cursor + ((text.codePointAt(cursor) ?? 0) > 0xffff ? 2 : 1));
   const last = text.charCodeAt(cursor - 1);
   const first = text.charCodeAt(cursor - 2);
   return Math.max(
      0,
      cursor - (last >= 0xdc00 && last <= 0xdfff && first >= 0xd800 && first <= 0xdbff ? 2 : 1)
   );
}

/** Apply an editing key; Enter submission and cancellation belong to the chat loop. */
export function editDraft(draft: Draft, key: Key): void {
   const { text, cursor } = draft;
   if (key.name === 'backspace' && cursor > 0) {
      draft.cursor = step(text, cursor, -1);
      draft.text = text.slice(0, draft.cursor) + text.slice(cursor);
   } else if (key.name === 'delete') {
      draft.text = text.slice(0, cursor) + text.slice(step(text, cursor, 1));
   } else if (key.name === 'left') {
      draft.cursor = step(text, cursor, -1);
   } else if (key.name === 'right') {
      draft.cursor = step(text, cursor, 1);
   } else if (key.name === 'home' || (key.ctrl && key.name === 'a')) {
      draft.cursor = 0;
   } else if (key.name === 'end' || (key.ctrl && key.name === 'e')) {
      draft.cursor = text.length;
   } else {
      const inserted = key.name === 'enter' ? '\n' : key.text;
      if (inserted !== undefined) {
         draft.text = text.slice(0, cursor) + inserted + text.slice(cursor);
         draft.cursor += inserted.length;
      }
   }
}
