# Early-route playtest checklist

Use this checklist with a current native build. It groups the awaiting-human
fixed candidates by route so one play session can validate several issues.
Only report what you directly observe. Leave any untested item open.

Run `mise run play-native`, then work through the sections in order. For audio
checks, use real speakers or headphones rather than a headless run.

## Boot and title flow

- [ ] #288 - Boot the intro through the jacket throw; no Kid or jacket sprite should leak at bottom-left before the throw.
- [ ] #210 - Rapidly skip intro, title, and menu with repeated Start presses; no black squares, corrupt Red Stealth/title art, or early gameplay release should appear.
- [ ] #254 - Rapidly skip intro/title/menu and watch the Red Stealth/title-screen art; it should not corrupt with the #210 route.
- [ ] #348 - Tap Start as soon as a title card is visible; it should enter the level immediately.

## Level 0x00 - Blue Lake Woods 1

- [ ] #211 - Walk Level 0 slopes in live play; slope traversal should not show wrong-facing or diagonal player frames.
- [ ] #372 - Hit forest prize blocks over tree/background art; the P/prize block should visibly wobble up/down instead of disappearing.
- [ ] #370 - Die to walking Dragon breath in Blue Lake Woods 1; after reload/continue, no stale fire should remain or damage Kid.

## Level 0x01 - Blue Lake Woods 2

- [ ] #155 - Jump near BLW2 Drips/slime and metal blocks; the slime should attach, drop, and fire in the ROM-like direction.
- [ ] #169 - Play through BLW2 Drips/slime encounters; placement, drops, crawlers, and attacks should look ROM-like.
- [ ] #213 - Reach the BLW2 Drips/metal-block corridor; slime, block mutation, and red projectile behaviour must match live evidence.
- [ ] #298 - Stand on a telepad; warp should happen only after the ROM countdown, not on quick walk-over.
- [ ] #331 - Walk through BLW2; RockTanks should no longer vanish at the wall near the teleporter.

## Levels 0x02-0x03 - Highwater Pass

- [ ] #065 - Enter a Shooter-block room; each visible Shooter block should fire on its own timer when it appears.
- [ ] #243 - In Highwater, watch Tar Monster flicker; the blue/black body should hold stable frames while alive.
- [ ] #325 - Enter Highwater 0x02/0x03; waterfall colours should cycle instead of freezing black or wrong.
- [ ] #185 - In Highwater Pass 2, approach a Hand; it should crawl/jump with solid Hand frames and still grab Kid on contact.

## Levels 0x04-0x05 - Under Skull Mountain

- [ ] #352 - Headbutt hidden jump-up blocks; the reveal should animate instead of popping.
- [ ] #333 - In USM and cave levels, Sphere should spin and EmoRock should not cycle shatter frames while alive.
- [ ] #323 - Throw Maniaxe axes in live play; the projectile should be spinning axe art, not a green dot.

## Level 0x07 - Isle of the Lion Lord

- [ ] #186 - Fight a Lion; hurt recoil, ledge leap, roar windup, and engagement range should behave like the ROM.

## Level 0x08 - Hills of the Warrior 1

- [ ] #350 - Enter Hills of the Warrior 1; lower trees and bushes should be green or brown, not blue.

## P0 named-level smoke

This check sits outside the numeric early route but belongs in the same P0
validation pass.

- [ ] #374 - Load Stairway to Oblivion, Bloody Swamp, Scorpion Isle, Lion's Den, and Hoverboard Beach; each should spawn playable with coherent level art, no instant death.

## Audio-device checks

- [ ] #339 - Load Under Skull Mountain 2; the level loading screen should be silent, with no weird/stale noise before gameplay.
- [ ] #342 - On real speakers, collect a clock prize; the clock tick should stop shortly after.
- [ ] #216 - On real speakers, break blocks and finish a level; SFX should trigger on time and not cut off late.
- [ ] #242 - On real speakers, bounce on rubber tiles; they should play the bounce/metal sound, not brick-break SFX.
- [ ] #287 - On real speakers, swing Samurai repeatedly; attack SFX should play once per swing without low/distorted overlap.
- [ ] #341 - On real speakers, spot-check audited SFX ids; native and ROM-backed GEMS should sound consistent.

## Report back

Copy one line per tested issue:

```text
#NNN PASS|FAIL - build/commit: <hash> - route: <level/action> - observed: <what happened>
```

For a failure, also provide the first bad frame or screenshot when possible.
Do not mark an issue passed from a different route or level than its checklist
item.
