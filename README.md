# Parallelization of 3D rendering

This repository contains my secondary school graduation thesis, which explores the implementation of various rendering techniques using Rust and WGPU. Through the included examples, readers can gain a deeper understanding of topics such as **GPU Driven Rendering** or **post-processing** effects.

## Documentation
 - While the documentation/paper for this thesis is only available in Czech, you're welcome to read it if you wish to
 - It is located in the file named `docs/Paralelizace_3D_renderovani.docx`

## Examples

The project is divided into multiple examples, each of which showcases a unique rendering technique. The first two examples were specifically created for educational purposes, while the rest demonstrate practical implementations of the rendering techniques.

 1. **triangle**
 2. **cube** (from WGPU)
 3. **gpu-driven-rendering**
 4. **post-processing**

The entire project is based on [WGPU GitHub examples](https://github.com/gfx-rs/wgpu/tree/master/wgpu/examples).

### How to run

If Rust is properly set up on your system, you can use the following command:

```
cargo run --example <example-name>
```

It is recommended to use the `--release` flag.

```
cargo run --release --example <example-name>
```

## Why?

The project was created to add new features to Dotrix, a 3D game engine written in Rust. While exploring rendering techniques can be a fun exercise in itself, the main purpose of this project is to enhance the functionality of Dotrix.

Dotrix website: https://dotrix.rs/

## Contacts

 - LinkedIn: [Oliver Řezníček](https://www.linkedin.com/in/oliver-reznicek)
 - Lowenware: https://lowenware.com/ 
