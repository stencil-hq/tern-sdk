/**
 * A `flow` surface: a running card with a spinner and an elapsed timer, log
 * lines streamed into it, a progress bar, then done.
 */
import { bar, connect } from '@stencil-hq/tern';

const steps = ['Resolving', 'Fetching', 'Compiling', 'Linking', 'Testing', 'Packaging'];
const STEP_MS = 450;

/** Waits `ms`. */
async function sleep(ms: number): Promise<void> {
   const { promise, resolve } = Promise.withResolvers<void>();
   setTimeout(resolve, ms);
   await promise;
}

const started = Date.now();
const session = await connect({ app: 'progress' });

if (!session) {
   for (const [i, step] of steps.entries()) {
      console.log(`${step}...`);
      await sleep(STEP_MS);
      console.log(`${bar((i + 1) / steps.length)} ${i + 1}/${steps.length}`);
   }
   console.log(`Done in ${((Date.now() - started) / 1000).toFixed(1)}s`);
} else {
   const surface = session.open({ mode: 'flow' });
   let stopped = false;
   const watch = (async () => {
      for await (const input of session) {
         if (input.type === 'key' && input.key.ctrl && input.key.name === 'c') stopped = true;
      }
   })();

   let log = '';
   const view = (done: number, took?: number) => (
      <card
         key="build"
         head="Building tern-sdk"
         status={took === undefined ? 'running' : 'done'}
         tone={took === undefined ? 'accent' : 'success'}
      >
         <row key="now" gap="md">
            {took === undefined ? (
               <spinner key="spin" style="dots" label={steps[done] ?? 'Finishing'} />
            ) : (
               <text key="spin">Finished</text>
            )}
            {took === undefined ? (
               <elapsed key="time" age={0} />
            ) : (
               <elapsed key="time" stopped={took} />
            )}
         </row>
         <ansi key="log" text={log} />
         <progress key="bar" value={done / steps.length} label={`${done}/${steps.length}`} />
      </card>
   );

   surface.render(view(0));
   for (const [i, step] of steps.entries()) {
      if (stopped) break;
      log += `\x1b[2m${new Date().toISOString().slice(11, 19)}\x1b[0m ${step}...`;
      surface.render(view(i));
      await sleep(STEP_MS);
      log += ' \x1b[32mok\x1b[0m\r\n';
      surface.render(view(i + 1));
   }
   const took = Date.now() - started;
   surface.render(view(steps.length, took));
   await session.close();
   await watch;
   console.log(stopped ? 'Stopped.' : `Done in ${(took / 1000).toFixed(1)}s`);
}
