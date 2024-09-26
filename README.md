
<p align="center">
<picture>
  <source width="300" media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/mororo18/draw/main/readme/logo-dark-mode.png">
  <source width="300" media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/mororo18/draw/main/readme/logo-light-mode.png">
  <img alt="draw">
</picture>
</p>

<h6 align="center">
  real-time renderer from scratch 
</h6>

## description
A CPU-based real-time renderer built in Rust, this project implements core graphics techniques like triangle rasterization, perspective projection, Phong shading, and texture mapping, rendering directly to the frame buffer via X11. It serves as a hands-on learning tool and a foundation for exploring efficient CPU-based software rendering.

### dependencies
install the latest version of [rust](https://www.rust-lang.org/tools/install) and:
```console
$ sudo apt install libx11-dev libxi-dev libxfixes-dev libxcursor-dev
```
### build and run
```console
$ git clone https://github.com/mororo18/draw.git
$ cd draw
$ cargo run --release
```

### screenshot

![alt text](https://github.com/mororo18/draw/blob/main/readme/airplane.png?raw=true)

### todo

- [x] perspective projection
- [x] triangle clipping
- [ ] camera movement by mouse input
- [x] phong shading
- [x] texture mapping
- [ ] wavefront .obj fileformat
- [ ] normal mapping
- [ ] decent user interface :(
- [ ] anti-alising
- [ ] vertical synchronization 
