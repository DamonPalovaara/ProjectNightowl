use web_rust::start::run;

fn main() {
    pollster::block_on(run());
}
