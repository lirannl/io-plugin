# IO-Plugin

IO-Plugin is a rust package which allows easily creating a plugin-system based on the following model:

1. The host spawns instances of its' plugins (by runnning their executables)
2. The host sends serialised messages on the plugin process' stdin
3. The host receives serialised responses on the plugin process' stdout

The plugins are intentionally NOT async, due to their single-use nature (each process must only be connected to a single host)

Theoretically, it is also possible to create plugins in other languages, though their interfaces will have to be determined manually. 
The messages are currently serialised using rmp-serde (this is subject to change at my discretion - though I expect to stick to serde-supported formats).

A usage example is available under ./io-plugins-test 

This package is currently WIP

Checklist:
- [x] Determine structure for translating a provided enum to the various relevant data structures.
- [x] Write a macro that converts said enum to the data structure
- [ ] Attribute-forwarding (besides just documentation)
- [ ] Create sensible default implementations (except for the plugin-trait methods)
