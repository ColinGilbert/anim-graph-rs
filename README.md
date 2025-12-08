This is an early WIP of an animation graph in Rust, using ozz-animation-rs to perform the actual playback/blending/etc.

It is not yet ready for use at all, but I'm working at it little by little every few days.

The structure is as follows:

- A graph definitions file is created by the user, usually with the assistance of some GUI tools.
- That graph definition is iterated over and used to create a runtime animation graph (or simply an animgraph.)
- The animgraph has no string lookups; everything is done by indexing into vectors.
- The end-user is encouraged to use the graph definition file to create a mapping for your parameters into their respective indexes.

MIT licensed.