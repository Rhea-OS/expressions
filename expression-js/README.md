# Expression JS integration

The expression engine is written in Rust using the `no_std` flags, meaning it can be cleanly ported to other platforms
with little effort. Naturally, JavaScript is somewhat special in this regard, making everyone's life harder. This
library aims to provide high-level bindings to the library through as WASM interface.

As the expression engine was written from scratch with the intent to run under WASM, in order to leverage Rust's highly
performant output code, it has very few dependencies.

## Obsidian-OS build system

The library integrates with Obsidian-OS's build system, through its `./build.js` file. It defines a list of independent
artifacts which can be built through a command-line interface. In an Obsidian-OS source environment, this package can be
build using an NPM command:

```bash
$ pnpm install
$ pnpm run build
$ # alternatively invoke the build command manually:
$ pnpm exec build build:main.wasm build:main.js -f
```

> [!important] In order for an integrated build such as above to be successful, the directory structure of the source
> code must be taken into account. As this library is technically
> a [Cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) member, it depends on the base
> expression engine through a relative path. If either of these packages are moved relative to each-other, the build may
> fail.

## Out-of-source build

If you wish to build this library without a full Obsidian-OS source tree (which is understandable), you can do so by
cloning the base expression engine.

If you attempt to `pnpm install` within the `expression-js` directory outside of an Obsidian-OS source tree, you will
likely run into errors that `../../builder` does not exist. This is of course expected. Hence, the example below is
completely manual.

```bash
$ cargo install wasm-pack
$ git clone https://github.com/Obsidian-OS/expressions.git
$ cd expressions
$ cargo update
$ cd expression-js
$ pnpm install esbuild
$ wasm-pack build --out build/mod
$ pnpm exec esbuild ./unit.ts --bundle --sourcemap --format=esm --loader:.wasm=binary --outdir=build/pkg
```

This produces the following important files:

1. `build/pkg/lib.js`: This is a standalone library which can be `import`ed into any existing library.
2. `build/mod/expression_js.d.ts`: Is the type definition for the library.
3. `build/mod/package.json`: This package.json file needs to be adjusted before it can be useful to the `lib.js` file,
   however this is left to the reader to determine the best way to do so.

When assembled into a module, yields a fully functional expression engine.

# Notes:

The JS bindings are mostly auto generated but also partially hand-written, there is a chance that source updates to the
base engine will break the bindings. If this is the case, you may need to rebuild from a clean slate by
1. Clearing build artifacts on 
   * The base engine
   * The bindings
   * The resulting bundle
2. Rebuild from scratch.

It is unfortunately not trivially possible to make cargo an npm interact when it comes to the build process. 
This makes it somewhat difficult to cleanly tie NodeJS imports and wasm artefacts together.
As a result, `package.json` specifies [subpath imports](https://nodejs.org/api/packages.html#subpath-imports) in order to simplify the linking process. 
Unfortunately, this means that the structure of build artifacts is more-or-less fixed. Please ensure that you use the correct build structure during manual buidls.
