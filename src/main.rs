use simul8::*;


fn main() {
    let res = pollster::block_on(simul8::run());

    if let Err(e) = res {
        util::show_error_dialog(&format!("Fatal error while running simul8: \"{:?}\"", e));
    }
}
