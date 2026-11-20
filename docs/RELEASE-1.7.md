# Cartographer 1.7: release notes

1.7 closes the owner's last item from the 1.6 panel check (ties in a yard ladder) and adds a yard end setting. Same gates as 1.6: tests written first, road parity 15/15, rail parity 8/8, fuel, CI on macOS and Linux, a fresh review by a separate model with one fix pass, live checks.

## What changed

- Yard ladders: the ladder's own ties now start where it leaves the main (they were looked for only at the ends of the ladder's long straight piece, so they began at its far end), each turnout on it gets lengthened switch ties, and each spur's own ties begin where it clears the ladder. The review found the first version laid doubled ties where spurs leave the ladder; fixed (test `a_spur_leaving_the_ladder_is_not_tied_twice`).
- `before_end` for yards (owner request).
- Bridges: hand-drawn road bridge deck; rail-bridge side walls per side.
- Panel and setup messages (#23, #25).

## Decisions taken (rulings), with what each costs if wrong

- Rail parity: Track Layer drew ladders bare, so in the station and terminal scenarios the ladder and spur tie objects are listed as fixed on purpose and left out of the comparison; every other object is still compared exactly, and the check fails if a listed object is no longer drawn. Cost if wrong: a regression in those tie objects is caught only by the unit tests.
- Fuel: a yard now draws more than Track Layer did (the ladder ties and switch ties); the yard case has its own limit, 1.7 times Track Layer (measured 1.643). Cost if wrong: a later slowdown of yards under that limit goes unnoticed.
- Lengthened switch ties are left out where a route's ties are drawn as one solid band (the tie budget is spent) and where a road or another track runs through.

## Deferred (minor findings from the review)

- The road bridge's side walls now also wobble and take the style's stroke factor and print minimum.
- A rail bridge over a yard uses the yard's stored spur counts even where the yard is too short and its spurs are left out, or where the road crosses outside the ladders.
- The parity script's `fixed` ids only match self-closing `<path .../>` elements.
