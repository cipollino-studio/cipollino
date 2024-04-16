
fn main() {
    cc::Build::new()
        .file("../../libs/curve_fit/c/intern/curve_fit_corners_detect.c")
        .file("../../libs/curve_fit/c/intern/curve_fit_cubic_refit.c")
        .file("../../libs/curve_fit/c/intern/curve_fit_cubic.c")
        .file("../../libs/curve_fit/c/intern/generic_heap.c")
        .compile("curve-fit-lib.a");
}
