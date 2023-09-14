use arbgeom_rs::run;
use pollster;

fn main() {
    pollster::block_on(run());
}