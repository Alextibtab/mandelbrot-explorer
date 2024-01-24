# Mandelbrot Explorer in Rust

This project is a Mandelbrot set explorer built using Rust, integrating `egui` for the user interface and `wgpu` for GPU acceleration. It utilizes WGSL shaders for rendering the Mandelbrot set.

## Overview

This simple project lets you to explore the Mandelbrot set, The project uses Rust and GPU shaders for real-time exploration.

## Features

- **Real-Time Exploration:** Zoom and pan through different areas of the Mandelbrot set.
- **High Performance:** Utilizes `wgpu` for GPU-accelerated rendering.
- **Customizable Render Settings:** Adjust iterations and other parameters.

## Requirements

- Rust (latest stable version)
- Graphics card supporting Vulkan, Metal, or DX12

## Getting Started

1. **Clone the repository:**

```bash
git clone https://github.com/Alextibtab/mandelbrot-explorer.git
cd mandelbrot-explorer
```

2. **Build and run the project:**
```bash
cargo run --release
```

## Usage

- Use the mouse to drag and pan around the fractal.
- Scroll to zoom in and out.
- Adjust the parameters in the UI to change the rendering of the Mandelbrot set.

## Example Images

Below are some example images generated using this Mandelbrot explorer. 

![Default View](/assets/default.png)
![Example Image 1](/assets/example-1.png)
![Example Image 2](/assets/example-2.png)
