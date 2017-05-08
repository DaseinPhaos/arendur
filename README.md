# arendur

[travis](https://travis-ci.org/DaseinPhaos/arendur.svg?branch=master)

Just Another Renderer. This time in Rust though.

This is a project guided by [pbrt](http://www.pbrt.org/). Its on `0.0.4` for the time being, with a functional path-tracing based renderer.

A sample scene rendered with 256 samples per pixel, using a Cornell Box modification created by [Guedis Cardenas and Morgan McGuire at Williams College, 2011](http://graphics.cs.williams.edu/data)):

![cornelbox](cbs256.png)



To tinker with it you can either clone the source code with `git`:

   ```sh
   $ git clone https://github.com/DaseinPhaos/arendur.git
   $ cd arendur
   ```

or simply grab it from [crates.io](https:://crates.io).

## What's next

Before 0.1, I want the module to support:

- [x] a console interface (implemented as `./examples/arencli.rs`)
- [x] area lights
- [ ] more materials
- [ ] a bidirectional path tracing based renderer

## Contributing

Contribution/guidance appreciated!

## License

This project is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE), [LICENSE-MIT](LICENSE-MIT) for details.
