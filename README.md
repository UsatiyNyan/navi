# navi

Serial Experiment #0

# TODO

**DONE: Phase 0 — The MVU spine, headless of graphics.** Build the loop backbone from the render-as-conductor sketch you already have: the runtime owns `Model` + a message queue (`Rc<RefCell<VecDeque<Msg>>>`), `dispatch` is a `Weak` upgrade-or-noop, and the loop drains → folds through `update`. No effects yet, no rendering yet. Test it with a trivial counter Model. *Done when:* you can dispatch `Increment` and watch a number change via a `println!`. This is the single most important phase — if this isn't clean, nothing above it will be. Your decision: the exact loop ordering (drain, then effects, then maybe-render).

**Phase 1 — Window + GPU clear.** Just `winit` + `wgpu`: open a window, get a device/queue/surface, clear it to a color every frame. No Model, no MVU, nothing. *Done when:* a colored window appears and survives resize. This exists only to prove your GPU init and the OS event loop work in isolation, before you entangle them with your architecture.

**Phase 2 — Wire the spine to the window.** Now marry Phase 0 and Phase 1: real `winit` events become `Msg`s (resize, close, pointer), and the redraw-request flag drives whether `render` runs this iteration. `render` still just clears the screen, but now it's `render(&Model, &mut render::State, ...)` reading real control state. *Done when:* an event-driven redraw — window only repaints when a message arrives, and genuinely idles otherwise (verify with a frame counter that a still window stops counting). This proves the event-driven-redraw decision you made pages ago.

**Phase 3 — Effects + one fake stream.** Add the effect layer: `update` returns effects, runtime runs them, they hold `Weak` dispatch. Write *one* effect that fakes a data stream — a timer dispatching `Tick` on an interval. No real data, no buffer yet; just prove the effect can dispatch repeatedly into the loop over time. *Done when:* `Tick`s arrive on their own and drive redraws, and pausing (a Model flag that stops the effect) makes them stop. This validates long-lived dispatch + teardown before real data complicates it.

**Phase 4 — Render *something* 3D.** First actual geometry: a hardcoded triangle, then a cube, with a fixed camera and a real VP matrix. This is a pure graphics phase — vertex/index buffers, a shader, a pipeline, `perspective_rh × look_at_rh`. No interaction, no Model-driven camera yet. *Done when:* a cube renders. This is where you climb the wgpu learning curve in isolation, uncontaminated by architecture.

**Phase 5 — Camera as control state.** Now connect Phase 4's matrix to the Model: camera lives in `render::Model`, pointer-drag `Msg`s move it, `render` builds VP from it each frame. Do **one** mode first (orbit — it's more constrained). *Done when:* dragging orbits the cube. This is Phase 5 of the *camera* sub-plan from my last message, folded in here — and the two-mode machine, the fly/orbit snap, the pivot question, and picking are all sub-steps *within* this phase, tackled in that order. Don't start them until single-mode orbit works.

**Phase 6 — The real buffer layer.** Replace the fake stream with your time-indexed store: `buffer::State` holds data, `buffer::Model` holds cursor/metadata/streams, effects resolve range-requests by mutating `buffer::State` and dispatching `SamplesArrived` notifications, and `render` reads the buffer + uploads dirty tails. *Done when:* real streamed data draws, and a `SamplesArrived` updates the view without passing bytes through `update`. This is the payoff phase — everything below it exists to make this clean.

**Phase 7 — The UI chrome layer.** *Now*, last, the `Element` tree — panels, toggles, legend, camera-reset button — the small diffed widget tree that `view` produces, composited over the 3D viewport. It's last because it's the least architecturally risky part and the one most decoupled from everything else. *Done when:* a button dispatches a `Msg` that changes the viz. Your decision here: how UI hit-testing coexists with 3D picking (which layer gets the pointer event first?).

Two things about the shape of this plan:

**Phases 0–3 contain zero interesting graphics and that's deliberate.** You prove the *architecture* turns over with fake data before real rendering can hide bugs in it. If you build the cube first (the temptation), you'll wrap an architecture around working graphics and the seams will be wrong — same mistake as building `view` before the loop.

**The natural stopping points are after Phase 3** (a live event loop with fake streaming — the whole runtime, no real content) and **after Phase 5** (interactive 3D — a real app, minus real data). Each is a demoable milestone. If motivation flags, those are where you rest.

So the very first concrete thing: Phase 0, window + clear. But before you touch code — one sequencing decision is *yours*, because it reveals whether you've internalized the core idea. I put "build the MVU spine" (Phase 1) *before* "render a cube" (Phase 4). Some people would swap them — cube first, it's more fun and more visible. Make the call for your own build and defend it: does the spine come before the cube, or the cube before the spine? Your answer tells me whether the architecture is load-bearing in your head yet or still decoration. Which order, and why?

## TODO: wasm host API and crate shape

- Treat `app` primarily as a library (host-controlled runtime), not as a required executable.
- Keep native CLI loop as an optional binary (`app-cli`) only for local debugging.
- For wasm integration, do not rely on `main` as the product entrypoint.
- Define exported host-facing state/callback API in phases:
  - `init`/`new` returns app state handle
  - `send` enqueues messages from JS/UI events
  - `tick`/`run_once` advances TEA runtime one turn
  - `snapshot`/`view` reads model-derived output for rendering
- Add wasm scheduling phase later (RAF/event callback wiring outside TEA core).
- Until that phase is done, wasm executable parity is intentionally not required.
