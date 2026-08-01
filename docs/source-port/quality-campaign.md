# Quality campaign: adopted and rejected techniques

Distilled from a 2026-08-01 research pass over comparable projects (accuracy
emulators, behaviour-matching source ports, decomp-scene tooling) and current
verification literature. Full raw reports live in the private KB vault, not
this repo. Each item below is either adopted (with its backlog row / gate
home) or explicitly rejected so the next session does not re-derive it.

## Adopted

1. **Julius-style scoped-predicate exceptions** for `oracle_limitation`.
   Julius's `save_compare.c` tolerates diffs only through predicates scoped by
   object type + field + value range + neighbourhood condition, each carrying
   a one-line reason class. That is the implementation shape for our
   exceptions once the count grows: never a whole advisory field, never an
   unscoped checkpoint class. `ADVISORY_FIELDS` stays empty. (Cite:
   bvschaik/julius `test/sav/sav_compare.c`.)
2. **devilutionX-style total, declarative checkpoint schema.** Their
   `memory_map` names every field a comparison covers, so "compared" vs
   "silently ignored" is auditable. Our checkpoint structs should keep that
   property: a new runtime field either enters the trace/comparison or is
   listed as deliberately untraced. (Cite: diasurgical/devilutionX
   `test/fixtures`, demo replay + field schema.)
3. **Gate-vacuity tests** (OpenRCT2 #11228 lesson: a replay gate that
   dry-ran for months). A test that injects a deliberate perturbation and
   asserts the TAS/pixel gate actually goes red. Phase 2 item; extends to
   `sometimes!`-style reachability assertions later.
4. **SingleStepTests 68000 conformance vectors** vs `r68k-emu`
   (`verify-cpu-vectors`): the CPU core under every timing claim is currently
   unvalidated; the vectors either indict or exonerate it cheaply. Ratcheted
   partial pass. (Cite: SingleStepTests/680x0, SingleStepTests/m68000; the
   MAME-derived set inherits MAME's bugs — a disagreement between the two
   sets is itself a finding.)
5. **verify-disasm-fidelity**: assemble `.cache/upstream/kid-chameleon-disasm`
   with AS and diff against ROM bytes — converts "the disassembly is truth"
   from assumption to gate. (Decomp-scene discipline: objdiff/asm-differ;
   asm-differ supports m68k.)
6. **Swarm scenarios in the sweep.** The 105-level sweep is idle-only — the
   single most suppressive configuration possible. Per-level seeded scenarios
   each omitting a random button subset find what idle suppresses; button
   relevance falls out as free triage input for `sweep-census`. (Groce et
   al., ISSTA 2012: +42% distinct fault modes over a hand-tuned config.)
7. **Metamorphic suite, oracle-free, in `verify-slice-fast`:**
   `advance(N) ≡ advance(k); advance(N−k)`; GEMS chunk-size invariance;
   save/rewind round-trip; no-op input padding shifts events exactly.
   Documented non-MR: horizontal mirroring (the ASM is not mirrored —
   different brake constants), written down so nobody builds it.
8. **Two-tier corpus framing.** Everything the loop agent can run is
   permission to commit; ratchet baselines advance only on evidence from a
   surface the loop cannot fit to (sealed composition scenarios, a second
   oracle's vote, conformance vectors). `oracle_injection_ratchet.py` is
   already the house pattern; generalise its shape, don't invent new ones.
9. **Second-oracle voting** (BlastEm is already buildable via
   `maybe_build_blastem`): `native ≠ jgenesis` alone is suspicion;
   `native ≠ both` is a bug; `jgenesis ≠ BlastEm` is an oracle bug to record,
   not chase. Mechanises most `oracle_limitation` citations. Longer-term.

## Rejected (with reasons, so they stay rejected)

- **Continuing the persistent-raster march/recurrence to the pixel horizon**
  — frozen and superseded by ADR 0002 (see it for the full cost argument and
  the #416 precedent).
- **Learned/fitted timing tables committed as constants** (DiffTune-class):
  fitting can reduce error with physically-wrong parameters; a fitted
  constant is committable only when independently derivable from the M68000
  manual/disassembly, else it is `// TODO(fidelity):`. Fitting remains useful
  for *ranking residuals* (where is the model structurally wrong), never for
  laundering guesses into green gates.
- **r68k cycle table adoption, mid-route GEMS seeding, general 68k timing
  model in the port** — closed in ADR 0001/#416; unchanged.
- **Global frame-local reconstruction as renderer fallback** — twice
  disproven (life icon 71→99); only the census-scoped exact variant is open
  (ADR 0002).
- **Naive golden-log worship**: matching a reference trace produces
  conformance to jgenesis, not to a Mega Drive (nesdev "Nintendulator
  emulators" warning). Provenance metadata (`verified_on`) belongs on golden
  fixtures; a second oracle is the structural fix.
