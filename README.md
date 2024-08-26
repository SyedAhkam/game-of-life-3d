# Conway's Game of Life 3D

Yet another [Conway's Game of Life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life) implementation however the canvas is located in a 3D world so that you could fly around and observe cells changing their state in realtime.

> Written in Rust using [Bevy](https://bevyengine.org).

## How to run

After installing Rust and fulfilling any [system dependencies](https://bevyengine.org/learn/quick-start/getting-started/setup/#installing-os-dependencies), you should be able to:

```sh
$ cargo run
```

## Customisation

In [`main.rs`](src/main.rs) there are a few constants which can be tweaked as it seems fit.

```rs
const TIME_STEP: u64 = 500; // in millis

const PLANE_SIZE: i32 = 48;
const CANVAS_SIZE: i32 = 32;
const CELL_SIZE: i32 = 4;
const CELL_GAP: i32 = 1;
const _CELLS_ON_CANVAS: i32 = (CANVAS_SIZE / CELL_SIZE).pow(2);

const CELL_ALIVE_COLOR: Color = Color::srgb(0.9, 0., 0.);
const CELL_DEAD_COLOR: Color = Color::srgb(0.9, 0.9, 0.9); // or use Color::NONE to make dead cells disappear
const PLANE_COLOR: Color = Color::srgb(0.3, 0.5, 0.3);
```
