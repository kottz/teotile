# 🪟 Teotile

Teotile is a simple game engine that can run simple pixel games. It was primarily designed to run on a 12x12 LED pixel grid using a WS281x LED. However, the code is portable enough to run on anything that can process input commands and output a grid of RGB pixels.

Teotile can be used alone or with a friend. There are both single-player and two-player games.

It is a useful software base for a *"Snake on your bookshelf"* or *"Flappy Bird on your Christmas tree"* style application.

https://github.com/user-attachments/assets/457638d9-5ba6-49b2-9d19-07be99ebfbb6

## Games

- **Connect Four** 
- **Tic-Tac-Toe**
- **Flappy Bird**
- **Snake**
- **Maze**
- **Space Invaders**
- **Doodle Jump**
- **Tetris**
- **Shooter**
- **Button War**
- **Wall Dodger**
- **Paint**

## Supported Platforms
- Raspberry Pi (Gamepad input + WS281x LED output)
- TUI
- Web ([demo](https://kottz.github.io/teotile/))
- WIP: Embedded on RPi Pico

## Usage
### Raspberry Pi
1. Connect WS281x LED strip to RPi, the default pin is 10 but it can be changed with the `--led-pin` argument.
2. Connect gamepad. Any gamepad that shows up in `/dev/input/js0` should work.
3. Clone the repository and run the program with `cargo run --release`.
```bash
git clone https://github.com/kottz/teotile
cd teotile/rpi
cargo run --release -- --led-pin 10
```

### TUI
```bash
git clone https://github.com/kottz/teotile
cd teotile
cargo run --release
```

### Embedded Rpi Pico (WIP)
Teotile uses the Embassy framework. Have a look at [their documentation](https://embassy.dev/book/#_getting_started) on how to get started with [probe-rs](https://probe.rs/). 

1. Connect the LED strip data pin to pin 16 on the Pico.
2. Flash Teotile to your RPi Pico:

```bash
git clone https://github.com/kottz/teotile
cd teotile/embedded
cargo run --release
```
