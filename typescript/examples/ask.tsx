/** `ask`: the "Which size?" form from the protocol book; prints the answer. */
import { createInterface } from 'node:readline';
import { ask, css, html } from '@stencil-hq/tern';

const sheet = css`
   .ask {
      display: flex;
      flex-direction: column;
      gap: 8px;
      max-width: 460px;
      padding: 10px 12px;
      border-radius: 8px;
      background: var(--card);
      box-shadow: inset 0 0 0 1px var(--l2);
   }
   .ask p {
      margin: 0;
   }
   .ask .sizes {
      display: flex;
      gap: 14px;
   }
   .ask :checked + span {
      color: var(--accent);
   }
   .ask button {
      align-self: flex-start;
      padding: 3px 12px;
      border-radius: 6px;
      background: var(--accent-fill);
      color: #fff;
   }
`;

const sizes: Readonly<Record<string, string>> = { s: 'Small', m: 'Medium', l: 'Large' };

const form = (
   <html.form key="ask" class="ask">
      <html.p key="q">Which size?</html.p>
      <html.div key="sizes" class="sizes">
         {Object.keys(sizes).map(value => (
            <html.label key={value}>
               <html.input key="r" type="radio" name="size" value={value} />
               <html.span key="t">{sizes[value]}</html.span>
            </html.label>
         ))}
      </html.div>
      <html.button key="go" actions={{ click: 'submit' }}>
         Create
      </html.button>
   </html.form>
);

/** Asks on plain stdin/stdout when Tern can't: the first line read, or null at end of input. */
async function askPlainly(): Promise<string | null> {
   process.stdout.write('Which size? (s/m/l) ');
   const lines = createInterface({ input: process.stdin, terminal: false });
   for await (const line of lines) {
      lines.close();
      return line.trim();
   }
   return null;
}

const answer = await ask(form, { css: sheet });
let picked: string | null = null;
if (answer?.type === 'unsupported') picked = await askPlainly();
else if (answer) {
   const size = answer.values['size'];
   picked = typeof size === 'string' ? size : null;
}
const label = picked === null ? undefined : sizes[picked];
console.log(label === undefined ? 'No size picked.' : `Picked ${label} (${picked}).`);
