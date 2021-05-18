# GNMX
This is our simulation for the course 'Computational biology'. 
// TODO
inspired by Hannes Svardal.

## Default values and configuring the simulation
We have used a default that resembles the parameters discussed in the paper. 

When in need for more precision, use the 'precision mode' at the top,
this will turn the sliders into input boxes which allow for more precision such as 0.001.

## Installing
This GitHub page contains precompiled releases for both Windows and Linux (as these are the only systems we have at our disposal). 
These can be found to the right of the GitHub page. Download the **GNMX.zip** file for your current OS. 
When unzipped, this folder contains a **backed.exe** on Windows just the **backend** on Linux.
There is also a **static** folder which contains the pre-compiled webassembly for the frontend.

```
To access the simulation perform the following steps:
1) double-click backend.exe in Windows or run ./backend in Linux
2) go to 'localhost:3030' in the browser. 
```

## Known bugs
As with most software, this simulation is not perfect. There were some edge cases we were not able to cleanly handle in time.
- Setting a population size smaller than the amount of patches results in a crash (as we divide by the patch size).
- In the same lines using very small population sizes causes the plots to look weird, we have not investigated if this is a bug or normal behaviour.
- Using extremely large numbers will cause an 'out of memory' crash.
- There are probably other configurations where a division by 0 might happen
- Switching between precision mode and default mode reset the values to their defaults.

## Development
This simulation is written in Rust. The program is divided into three crates (or packages). 
The frontend code, the simulation itself and the server logic.

Compiling this project for your own machine requires a Rust compiler. Use ``cargo run`` to do so.
How to install rust can be found [here](https://www.rust-lang.org/tools/install)

This project uses [wasm-pack](https://github.com/rustwasm/wasm-pack) to compile our Rust frontend code to webassembly.
This can be installed with ``cargo install wasm-pack`` command.
Use the command ``build --target web --out-name wasm --out-dir ../static`` in the frontend folder to generate the webassembly and additional files.
Be sure to do this before running, otherwise the webassembly will not be found.

Add ``RUST_LOG=info`` to your environment variables to get useful insights in the program execution.
Use ``RUST_LOG=debug`` to see everything that is happening.