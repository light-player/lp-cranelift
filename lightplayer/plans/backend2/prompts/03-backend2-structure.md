I have moved it to 00-overview, as this is more informational then actionable.

Next, I want to talk about the folder and file structure of the new backend.

It will go into lightplayer/crates/lp-glsl/src/backend2

It will completely replace lightplayer/crates/lp-glsl/src/backend. We may copy over code from the old one, but this a full replacement.

The main concepts provided by the backend should likely be:

- concept of target (arch, flags, module type, etc)
- module code mentioned in the overview
- transform pipeline
- building the executable (that is defined in exec/) 

The consumers of this code are:
- the frontend (for generating the module)
- the wrapper (which we haven't really separated, but it would be what ties everything together)


I want the code to be well structured, and specifically, I want the transform logic
to be generic so that we can implement more transforms (other than fixed32) in the future.

Please critique this plan. Look at frontend, exec, and backend2. What should we include?
What should we exclude?
What should be moved form other layers?