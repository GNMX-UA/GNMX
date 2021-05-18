# GNMX
This is our simulation for the course 'Computational biology'. 
// TODO
inspired by Hannes Svardal.

This simulation is written in Rust. The program is divided into three crates (or packages). The frontend code, the simulation itself and the server logic.

## Installing
// TODO


## development
Compiling this project for your own machine requires a Rust compiler. Use ``cargo run`` to do so.

Add ``RUST_LOG=info`` to your environment variables to get useful insights in the program execution.
Use ``RUST_LOG=debug`` to see everything that is happening.

This project uses [wasm-pack](https://github.com/rustwasm/wasm-pack) to compile our Rust frontend code to webassembly.
Use the command ``build --target web --out-name wasm --out-dir ../static`` in the frontend folder to generate the webassembly and additional files.