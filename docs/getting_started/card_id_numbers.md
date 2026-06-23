# Card ID numbers

Whenever a file of any type is loaded by Gitronics during a `build` command, Gitronics will copy and paste the cards into a single model.
If two different files share the same card ID number, for example, two different filler models sharing a surface, Gitronics will include that surface twice in the assembled model, which will cause a validation error, crashing the build process.
Gitronics will warn about which card IDs are duplicated with an error message.

[Migjorn](https://github.com/Fusion4Energy/migjorn), the parser used internally by Gitronics, is capable of renumbering on the fly any card ID.
However, as a design choice, Gitronics does not perform any automatic renumbering of card IDs. 
This is to avoid unexpected changes in the assembled model that could be difficult to track and debug.

Therefore, it is the responsibility of the user to ensure that all card IDs are unique across all files in the project.
The recommended practice is to keep a consistent numbering scheme for all the individual files in the project, so that they can be assembled without conflicts.
Tools like [Migjorn](https://github.com/Fusion4Energy/migjorn) or [F4Enix](https://github.com/Fusion4Energy/F4Enix) make the renumbering of MCNP files effortless.

!!! tip "A single card ID per system"
    A good practice is to assign the same card ID number to the different cards related to the same system. That is, make the universe ID of a filler model the same as the first cell ID, which is also the first surface ID, the first material ID (if the system needs non-standard materials). Same with tallies and transformations that are specific to that system. 