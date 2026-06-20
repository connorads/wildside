# Why Wildside exists

Two things at once, really. A childhood game, and an excuse to test something.

I grew up playing Kid Chameleon with my family on the Mega Drive. It was brutally
hard, never gave you a break, and I never finished it. Wildside is a native Rust
reimplementation of that game. I do want it playable. But that was never the whole
point. The rest of it is to see what happens when you aim coding agents at a hard
problem, let them run in a loop, and give them a way to check their own work that
they cannot fudge.

## Why Kid Chameleon

Nostalgia aside, it makes a good test subject. It is a fixed, deterministic 68k
game with a complete public disassembly, so there is a written source of truth for
almost every behaviour. It also runs in an accurate emulator, so the port can be
compared against the original frame by frame and byte by byte.

That combination is rare. Most software has nothing to check against. Here, "is
this correct?" has an objective answer for nearly any question I can ask. That is
exactly what I wanted before letting an agent loose on a problem without me grading
every step.

## The verification spine

Most of the effort went here, and it is what made the rest work.

- **The oracle.** `jgenesis` runs the real ROM. The port is compared against it on
  state, rendered frames, and audio. Disagree, and the port is wrong until proven
  otherwise.
- **The evidence ladder.** Progress carries explicit labels (stubbed, boots,
  replay-matched, branch-matched, mechanic-swept, human-play-stable), so "done"
  never claims more than the evidence supports. A replay matching the oracle is a
  long way from a human finding the game stable to play, and the labels keep that
  honest.
- **The TAS frontier.** A tool-assisted speedrun is a long recorded sequence of
  inputs. Replay it, count how many frames stay matched to the oracle, and you get
  one number that climbs as the port gets more accurate. Closing the frontier means
  the whole route replays clean. The oracle decides, so the number is hard to fake.
- **The sweep.** All 105 levels run against the oracle and their divergences get
  categorised, so a fix that helps level 1 and breaks level 23 shows up the same
  day.
- **Bugs are features.** The original's bugs are reproduced on purpose and marked
  in the code. A faithful port has to be faithful to the mistakes too.

## What I tried, and what stuck

The first version leaned hard into autonomy. Agents ran in a continuous loop,
"Ralph loop" style: re-prompt, let it go. Goal contracts wrote the target state out
as JSON, and a cascade was meant to chase those goals on its own. A few goal files
still sit in the tree as fossils (`docs/source-port/level-1-goal.json` and
friends).

It mostly did not work the way the pitch promised. A loop with nobody steering it
drifts. It grinds the wrong thing, declares victory on thin evidence, or polishes a
corner nobody asked about. So I deleted the autonomous orchestrator (you can watch
it go in the git history, around the "autonomy demolition" commits). What replaced
it is duller and works better: a human-ordered backlog and a fixed loop prompt. I
decide what matters next, an agent does the investigation and the work against the
oracle, and every fix has to survive the regression gate before it counts.

The thing that made any of it work was never the autonomy. It was the oracle and
the small, checkable steps. When "right" is machine-checkable, an agent can grind
on a hard problem for a long time and actually get somewhere. When it is not, it
wanders off. If I take one finding from this, that is it.

## Honest scope

This is a hobby experiment, not a study. No controls, no benchmarks, no grand
claim. A game I loved as a kid, used as a stress test for a workflow I found
interesting to build. The port will not be perfect and is not trying to be. If it
turns out fun to play, and these notes help someone doing something similar, that
is plenty.

## Read more

- How the port is done, in depth: [`../CLAUDE.md`](../CLAUDE.md) and
  [`source-port/methodology.md`](source-port/methodology.md)
- The loop a session actually follows: [`source-port/loop.md`](source-port/loop.md)
- What is still being learned, and where to source it:
  [`source-port/research-map.md`](source-port/research-map.md)
