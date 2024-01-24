mod ui;

use mandelbrot_explorer::run;

fn main() {
    pollster::block_on(run());
}
