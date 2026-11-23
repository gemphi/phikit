# The botanist: what a person does, tested

Each step is tested in `src/test.rs`, against Apple's scenes where they
are on this disk.

1. The window opens on the creator: the title, Head / Body / Backpack,
   Type, Eye Shape, Material, Material Color, Light Color, the robot
   standing idle, and Start Planting.
2. A press on `head3` and on `metal` gives the head that shape and that
   material; a press on Body shows the body's shapes, and no Eye Shape.
3. Start Planting shows the greenhouse, which says "Loading assets…"
   while Apple's scene is read.
4. Once read, the greenhouse shows its three planters (coffeeBerry,
   poppy, yucca), the score at 0, Replay and Exit Game.
5. A press on each planter walks the robot there and propagates it; with
   all three, the score says "Awesome job!" and "You propagated 3 plants".
6. Exit Game goes back to the creator.
