// Real-time CDP verifier for the scroll animation.
//
// The screenshot and --virtual-time-budget harnesses cannot measure this:
// requestAnimationFrame is starved under virtual time (measured: 0 frames
// granted), and a screenshot fires before or long after the animation. This
// attaches to a real Chrome, waits for the page, then reads scrollY in a loop
// while the animation runs, so the numbers are wall-clock real.
//
// Usage: node scripts/verify-scroll.mjs <pageUrl> <cdpPort>

const [, , url = "http://127.0.0.1:3000/scrolltest.html", port = "9222"] = process.argv;

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function findTarget() {
  for (let attempt = 0; attempt < 60; attempt++) {
    try {
      const res = await fetch(`http://127.0.0.1:${port}/json/list`);
      const list = await res.json();
      const page = list.find((t) => t.type === "page" && t.webSocketDebuggerUrl);
      if (page) return page;
    } catch {
      /* Chrome not up yet */
    }
    await sleep(200);
  }
  throw new Error("no debuggable page found");
}

const target = await findTarget();
const ws = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((resolve, reject) => {
  ws.addEventListener("open", resolve, { once: true });
  ws.addEventListener("error", reject, { once: true });
});

let nextId = 1;
const pending = new Map();
ws.addEventListener("message", (event) => {
  const msg = JSON.parse(event.data);
  if (msg.id && pending.has(msg.id)) {
    pending.get(msg.id)(msg);
    pending.delete(msg.id);
  }
});

function send(method, params = {}) {
  const id = nextId++;
  return new Promise((resolve, reject) => {
    pending.set(id, (msg) => (msg.error ? reject(new Error(msg.error.message)) : resolve(msg.result)));
    ws.send(JSON.stringify({ id, method, params }));
  });
}

async function evaluate(expression) {
  const r = await send("Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true });
  if (r.exceptionDetails) throw new Error(r.exceptionDetails.text || "eval failed");
  return r.result.value;
}

await send("Page.enable");
await send("Runtime.enable");

// Navigate only if we are not already on the target.
const here = await evaluate("location.href");
if (here !== url) {
  await send("Page.navigate", { url });
}

// Wait for app.js to have run.
for (let i = 0; i < 100; i++) {
  if (await evaluate("!!document.querySelector('.to-top') && !!document.getElementById('mark')")) break;
  await sleep(100);
}
await sleep(300);

const report = await evaluate(`(async () => {
  const wait = (ms) => new Promise((r) => setTimeout(r, ms));
  const out = [];

  // Count frames granted over 200ms - proves rAF is alive in this harness.
  let frames = 0;
  let stop = false;
  (function tick(){ if (stop) return; frames++; requestAnimationFrame(tick); })();
  await wait(200);
  stop = true;
  out.push('rAF frames in 200ms: ' + frames + '  (0 means the harness cannot measure this at all)');

  async function profile(label, setup) {
    await wait(250);
    setup();
    await wait(250);
    const t0 = performance.now();
    const samples = [];
    return new Promise((resolve) => {
      const iv = setInterval(() => {
        samples.push(Math.round(window.scrollY));
        if (performance.now() - t0 > 900 || window.scrollY === 0) {
          clearInterval(iv);
          out.push(label + ': ' + samples.join(' -> '));
          resolve();
        }
      }, 40);
    });
  }

  await profile('from 5200px, click to-top', () => {
    window.scrollTo(0, 5200);
    document.querySelector('.to-top').click();
  });

  await profile('from 3800px, click to-top', () => {
    window.scrollTo(0, 3800);
    document.querySelector('.to-top').click();
  });

  await profile('anchor click (heading 2080px down)', () => {
    window.scrollTo(0, 0);
    document.getElementById('top-link').click();
  });

  out.push('final scrollY: ' + Math.round(window.scrollY) + '  hash: ' + (location.hash || 'none'));
  out.push('scroll-behavior: ' + getComputedStyle(document.documentElement).scrollBehavior);
  return out.join('\\n');
})()`);

console.log(report);
ws.close();
