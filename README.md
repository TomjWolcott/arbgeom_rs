# Arbgeom rendering prototype
This was a prototype I made to work out the fundamental idea behind the [arbgeom game](https://tomwol.itch.io/arbgeom).

A lot has changed in the arbgeom game since this, the geometry/manifold builder is an example of this.  The main idea behind the rendering is really the only thing that is the same as in arbgeom.  I am planning on making public the current geometry builder when fully implemented, so people will be able to create their own levels for arbgeom with docs.

## Movement
WASD, Space, Shift for movement and drag with the mouse to turn the camera.
You can also add kebinds to some of the parameters of manifolds, for example the one that is currently set up uses t/g, y/h, u/j to control the various radii of the ditorus

## Screenshots
Inside the surface of the ditorus
------
<img width="1063" alt="Screenshot 2024-05-05 at 11 09 12" src="https://github.com/TomjWolcott/arbgeom_rs/assets/134332655/cb6f8d05-582a-4f2e-8cb8-c7433888e160">

Inside the surface of a hypersphere (to do this change the code: uncomment hypersphere and comment ditorus)
--------
<img width="924" alt="Screenshot 2024-05-05 at 11 12 30" src="https://github.com/TomjWolcott/arbgeom_rs/assets/134332655/2ccf87d5-7f63-4624-8789-5ae3b7cea411">

Inside the surface of an extruded sphere (uncomment extruded sphere and comment hypersphere)
-------
<img width="1003" alt="Screenshot 2024-05-05 at 11 14 33" src="https://github.com/TomjWolcott/arbgeom_rs/assets/134332655/a903c6fb-0e0d-4c29-b25b-0f7b7b1f1766">
