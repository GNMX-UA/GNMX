# GNMX
This is the project of Thomas Dooms, Ward Gauderis, Kato Starckx and Lauren Van Nimmen for the course 'Introduction to computational biology' at the University of Antwerp. 
This forward-time simulation was made with the purpose of investigating the genetic polymorphism phenomenon.
The model is based on the [paper](https://doi.org/10.1016/j.tpb.2014.11.002) by Hannes Svardal, Claus Rueffler and Joachim Hermisson.
More information can be found in the [report](report.pdf) (in Dutch).

## Configuring the simulation
When in need for more precision, use the 'precision mode' at the top,
this will turn the sliders into input boxes which allow for more precision such as 0.001.

The graphs will only plot a subset of the simulated data to keep the simulation real-time.
They will automatically remove data after 30 000 ticks, this can be turned off with the 'forget' option.

## Manual Installation
This GitHub page contains precompiled releases for both Windows and Linux (as these are the only systems we have at our disposal, a Docker container is also available [here](https://hub.docker.com/repository/docker/wardgauderis/gnmx)). 
These can be found to the right of the GitHub page. Download the **GNMX.zip** file for your current OS. 
When unzipped, this folder contains a **backed.exe** on Windows just the **backend** on Linux.
There is also a **static** folder which contains the pre-compiled webassembly for the frontend.

```
To access the simulation perform the following steps:
1) double-click backend.exe in Windows or run ./backend in Linux
2) go to 'localhost:3030' in the browser. 
```

## Docker
It is possible to run the server as a [docker container](https://hub.docker.com/repository/docker/wardgauderis/gnmx). Help on installing docker can be found 
[here](https://docs.docker.com/get-docker/). 
Deploying and starting the server can be done with the following command (assuming docker is installed).

```
docker run -d -p 3030:3030 wardgauderis/gnmx
```

After this is done, the webpage will be accessible in 'localhost:3030' in the browser.


## Known bugs
There are some edge cases we were not able to cleanly handle in time.
- Setting a population size smaller than the amount of patches results in a crash (as we divide by the patch size).
- Plots throw away data after 30 000 ticks, when the simulation only has very few individuals only a select amount data points wil be generated every 30 000 ticks. 
  This will cause weird graphs with only a few values.
- Switching between precision mode and default mode resets the values to their defaults.

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
