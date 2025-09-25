# Survicraft Examples

Examples and demos for how to use the Survicraft library. These examples cover
a variety of isolated test cases. This is mainly a way to test each feature
from the game for development purposes.

### Examples

**[Character Controller](character.rs)** is a simple example of the character
controller used in Survicraft. It includes basic movement, jumping, and
collision detection. You can move around using the WASD keys and jump with the
spacebar. This also includes an experimental mode for enabling dynamic physics.
This mode enables the physic character controller, which is a bit worse when
doing multiplayer, but it can be useful for other games, so I kept it here.

**[Terrain](terrain.rs)** demonstrates how to generate terrain and walk around
using the character controller. The terrain is generated using noise functions,
then we build a mesh for each chunk. You can move around using the WASD keys
and jump with the spacebar.
