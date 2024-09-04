
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
A CPU-based real-time renderer built from the ground up in Rust, this project explores the core concepts of graphics programming by implementing essential rendering techniques without relying on external graphics APIs. The renderer supports triangle rasterization, perspective projection, Phong shading, and texture mapping, all rendered directly to the frame buffer using the X11 API. This project is designed to be both a hands-on learning experience and a strong foundation for further exploration into efficient software rendering techniques.

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
