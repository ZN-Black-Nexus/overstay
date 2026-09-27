# Overstay — Game Design

Co-op (and solo-viable) 3D horror game about a contracted cleanup crew sent
into "attention-sick" properties: locations where sustained observation —
staring, lingering, backtracking — causes the space (and the people in it)
to react.

## Core loop: Fixation / Dread

- A **Fixation** value builds from player behavior: staring at one spot,
  backtracking through cleared rooms, holding a light on something too long,
  standing still in the dark.
- A **global Dread meter** tracks the whole match. It rises from unresolved
  possessions and stays elevated — it does not passively decay just because
  time passes.
- The crew's job is to contain reactions before Dread crosses the threshold
  that makes everything worse at once, then extract.

## Possession

Anything that absorbs enough Fixation can be possessed — a player, or a
piece of the environment.

- **Player possession**
  - Starts subtly: input starts "sticking" (letting go of forward still
    drifts you a moment longer, turns overshoot). Escalates exponentially
    the longer it runs.
  - Late-stage: the possessed player loses voice entirely (can't warn
    teammates, even face to face).
  - Tell for other players: proximity chat distortion — a possessed
    player's voice is garbled/delayed/doubled, but only to people standing
    near them. Proximity chat is a real gameplay signal, not flavor.
  - Cured with a non-lethal tool (placeholder item: a crucifix, no unique
    animation yet — swap once the story lands on something more specific).
    Channel it on the suspected player for a few seconds while adjacent.
    Limited charges. No harm if you're right; if you're wrong, you've
    wasted a charge standing next to whatever the real threat is.
- **Environmental possession**
  - A wall, cabinet, or stretch of floor that's absorbed enough Fixation
    cracks open and a humanoid shape tears free — a real, temporary,
    corporeal threat (not a patrolling AI), leaving a permanent scar/hole
    in the environment afterward.
  - Cured/repelled the same way conceptually as player possession feeds
    into the shared system, but is dealt with physically (stunned/fought
    off), not via the cure tool.

### Escalation tiers (per possession left unresolved)

1. **Possession** (recoverable) — control drift, then mute. Fixed with the
   cure tool, no harm done.
2. **Full possession / soul-cast-out** (more common, still recoverable-ish)
   — if unresolved too long: the character stun-locks, shakes, drops to
   their knees, head raised — soul forced out. The body becomes a slow but
   learning NPC threat that can *only* be put down with the sidearm (the
   cure tool no longer works — nothing left inside to restore).
   - The ejected player becomes a **spectator**, not eliminated. If a
     spare/unclaimed body exists somewhere in the generated map, they can
     repossess it and rejoin play. Spare vessels are rare but not nonexistent
     — roughly on the order of 1 per ~4 players (a 4-player match usually has
     one, a 6-player match maybe two). Exact ratio is a tuning knob.
   - While spectating, a player can nudge small environmental things
     (flicker a light, slam a door, topple an object) to assist the living
     crew — but doing so adds to the global Dread meter, same as any other
     possession. It's a real trade-off, not a free assist.
3. **Abomination breakout** (rare, catastrophic) — permanent death for that
   character (no repossession), and a real, unstoppable monster spawns.
   Re-arms the possession system: another player can be possessed later in
   the same match.
   - Cap on simultaneous possessions: `players - 1`, so at least one player
     is always unpossessed. A 2-player match can only ever have one
     possessed at a time; 3+ players opens a rare chance of two
     simultaneously.

### Solo mode

No player-possession loop (nothing to deceive or confirm with no one else
around). Solo leans fully into the two "objective" horror layers:
environmental possession, and — if personal Fixation runs unchecked — a
real stalking abomination shows up directly. Pure survival-and-hide instead
of survival-and-suspicion.

## Tools

- **Cure tool** (placeholder: crucifix, no animation yet) — non-lethal,
  channel near a suspected possessed player, limited charges. Restores a
  Tier 1 possession only.
- **Sidearm** — dual purpose. Stuns/staggers the Tier-3 abomination
  (temporary stagger, never kills it — enough to break a chase or buy a
  barricade window). Can also injure or kill *other players* if hit: a few
  hits to down someone, not a one-shot kill, with a visible blood trail on
  hit (griefing is possible, and the blood trail doubles as a tracking
  signal for finding a wounded teammate or the shooter).

## Coins & progression

- Coins are personal, not shared: found as physical pickups in the world.
- Extraction also grants a small, flat amount of coins (safe, guaranteed
  income vs. the riskier bonus of scavenging).
- Looking for coins means wandering off the safe path — which itself feeds
  Fixation. Greed has a real, systemic cost.
- Rare **fake coins**: visually indistinguishable from real ones. Picking
  one up triggers a harmless-but-scary hallucination that can mimic the
  *same tells* used to spot a real possession (e.g. garbled proximity-chat
  voice) — deliberately blurring real evidence with false positives. Not
  frequent.
- Coins buy cosmetic unlocks (see below). Purely cosmetic — no gameplay
  effect (nothing that blocks vision, changes hitbox, etc.).

## Cosmetics

- Modular character customization: hat, hair, shirt, pants, etc.
- Unlocked via achievements, or bought with in-run coins.
- Later (post-launch, on Steam): coins purchasable with real money. The
  in-game coin ledger should be built as its own internal system now so a
  real-money source can be plugged in later without rework.

## Story

You work for a company that handles "attention-sick" properties — places
where something happened (mass trauma, a ritual, an accident) that left
them able to react to sustained observation. The job: go in, use
company-issued gear to keep the site's reaction contained, and extract
before it escalates past containment. Framing still needs tightening
(the company itself, why this specific crew), but the contractor/cleanup
premise is locked.

## First map

**The Backrooms** — liminal, repetitive geometry (yellow walls, damp
carpet). Deliberately chosen as the first map because it needs no bespoke
level design and its disorienting sameness reinforces the Fixation theme
(easy to lose track of where you've been, easy to linger without noticing).

## Tech stack

- **Engine**: Godot 4.7 + Rust via [gdext](https://github.com/godot-rust/gdext)
  (`godot` crate) — gameplay logic in Rust, Godot's editor/renderer/physics
  underneath. Not Bevy.
- **Networking**: not yet decided. Candidates on the table:
  - Steamworks Networking Sockets (same approach as Lethal Company/R.E.P.O.
    — Steam's relay handles NAT traversal, host shares an invite/lobby).
  - Self-hosted relay + WebRTC/QUIC hole-punching (platform-independent,
    not tied to Steam, more infra to maintain).
  - A hybrid: abstract the netcode behind a trait, ship Steamworks first,
    leave room for a relay backend later.

  A blockchain-based connection gateway was considered and ruled out —
  blockchains solve distributed consensus on shared state, not low-latency
  signaling/NAT traversal, so it doesn't address the actual problem.
