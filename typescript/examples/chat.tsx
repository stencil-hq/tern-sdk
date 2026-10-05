/**
 * An `inline` surface: a transcript in `main` and an `editor` in `dock` the
 * program edits from keys. Enter sends, Escape quits.
 */
import { createInterface } from 'node:readline';
import { connect, type Key } from '@stencil-hq/tern';
import { editDraft } from './chat-draft.js';

interface Message {
   readonly id: number;
   readonly who: 'you' | 'tern';
   readonly text: string;
}

const GREETING = 'Type a message and press **Enter**. **Escape** quits.';

/** The reply to a message. */
function reply(text: string): string {
   const words = text.trim().split(/\s+/).length;
   return `You said *${text.trim()}* (${words} word${words === 1 ? '' : 's'}).`;
}

const session = await connect({ app: 'chat' });

if (!session) {
   console.log(GREETING.replaceAll('**', ''));
   const lines = createInterface({ input: process.stdin, output: process.stdout, prompt: '> ' });
   lines.prompt();
   for await (const line of lines) {
      if (line.trim() === '/quit') break;
      if (line.trim() !== '') console.log(`tern: ${reply(line).replaceAll('*', '')}`);
      lines.prompt();
   }
   lines.close();
} else {
   const surface = session.open({ mode: 'inline', title: 'Chat' });
   const messages: Message[] = [{ id: 0, who: 'tern', text: GREETING }];
   const draft = { text: '', cursor: 0 };

   const render = (): void =>
      surface.render({
         main: (
            <col>
               {messages.map(message => (
                  <card
                     key={`m${message.id}`}
                     head={message.who}
                     tone={message.who === 'you' ? 'user' : 'neutral'}
                  >
                     <md key="t">{message.text}</md>
                  </card>
               ))}
            </col>
         ),
         dock: (
            <col>
               <editor
                  key="ed"
                  text={draft.text}
                  cursor={draft.cursor}
                  placeholder="Message"
                  prompt="> "
               />
               <status key="bar">
                  <seg key="n" icon="message">{`${messages.length} messages`}</seg>
                  <seg key="keys" side="right">
                     Enter sends, Esc quits
                  </seg>
               </status>
            </col>
         ),
      });

   /** Applies one key to the draft; false when it quits. */
   const edit = (key: Key): boolean => {
      if (key.name === 'escape' || (key.ctrl && (key.name === 'c' || key.name === 'd')))
         return false;
      if (key.name === 'enter' && !key.shift && !key.alt) {
         if (draft.text.trim() !== '') {
            messages.push({ id: messages.length, who: 'you', text: draft.text });
            messages.push({ id: messages.length, who: 'tern', text: reply(draft.text) });
         }
         draft.text = '';
         draft.cursor = 0;
      } else editDraft(draft, key);
      return true;
   };

   render();
   surface.focus('dock.ed');
   for await (const input of session) {
      if (input.type === 'key' && !edit(input.key)) break;
      render();
   }
   await session.close();
}
