//! Planetarium crate integration tests

use planetarium::{Canvas, ImageFormat, SpotShape, Window};

/// Creates a 64x64 canvas image for all tests.
fn mkimage() -> Canvas {
    let mut c = Canvas::new(64, 64);
    c.set_background(1000);

    let shape = SpotShape::default().scale(4.5);
    let shape2 = shape.stretch(1.7, 0.7).rotate(45.0);

    c.add_spot((17.6, 15.2), shape, 0.9);
    c.add_spot((45.8, 33.5), shape2, 0.5);

    c.draw();
    c
}

#[test]
fn export_raw8bpp() {
    let img = mkimage().export_image(ImageFormat::RawGamma8Bpp).unwrap();

    // std::fs::write("tests/test_8bpp.raw", &img).unwrap();
    let golden_img = include_bytes!("test_8bpp.raw");
    assert_eq!(img, golden_img);
}

#[test]
fn export_window_raw8bpp() {
    let wnd = Window::new(32, 16).at(5, 8);

    let img = mkimage()
        .export_window_image(wnd, ImageFormat::RawGamma8Bpp)
        .unwrap();

    // std::fs::write("tests/test_wnd_8bpp.raw", &img).unwrap();
    let golden_img = include_bytes!("test_wnd_8bpp.raw");
    assert_eq!(img, golden_img);
}

#[test]
fn export_sub_raw8bpp() {
    let img = mkimage()
        .export_subsampled_image((2, 2), ImageFormat::RawGamma8Bpp)
        .unwrap();

    // std::fs::write("tests/test_sub_8bpp.raw", &img).unwrap();
    let golden_img = include_bytes!("test_sub_8bpp.raw");
    assert_eq!(img, golden_img);
}

#[test]
fn export_raw10bpp() {
    let img = mkimage()
        .export_image(ImageFormat::RawLinear10BppLE)
        .unwrap();

    // std::fs::write("tests/test_10bpp.raw", &img).unwrap();
    let golden_img = include_bytes!("test_10bpp.raw");
    assert_eq!(img, golden_img);
}

#[test]
fn export_window_raw10bpp() {
    let wnd = Window::new(32, 16).at(5, 8);

    let img = mkimage()
        .export_window_image(wnd, ImageFormat::RawLinear10BppLE)
        .unwrap();

    // std::fs::write("tests/test_wnd_10bpp.raw", &img).unwrap();
    let golden_img = include_bytes!("test_wnd_10bpp.raw");
    assert_eq!(img, golden_img);
}

#[test]
fn export_sub_raw10bpp() {
    let img = mkimage()
        .export_subsampled_image((2, 2), ImageFormat::RawLinear10BppLE)
        .unwrap();

    // std::fs::write("tests/test_sub_10bpp.raw", &img).unwrap();
    let golden_img = include_bytes!("test_sub_10bpp.raw");
    assert_eq!(img, golden_img);
}

#[test]
fn export_raw12bpp() {
    let img = mkimage()
        .export_image(ImageFormat::RawLinear12BppLE)
        .unwrap();

    // std::fs::write("tests/test_12bpp.raw", &img).unwrap();
    let golden_img = include_bytes!("test_12bpp.raw");
    assert_eq!(img, golden_img);
}

#[test]
fn export_window_raw12bpp() {
    let wnd = Window::new(32, 16).at(5, 8);

    let img = mkimage()
        .export_window_image(wnd, ImageFormat::RawLinear12BppLE)
        .unwrap();

    // std::fs::write("tests/test_wnd_12bpp.raw", &img).unwrap();
    let golden_img = include_bytes!("test_wnd_12bpp.raw");
    assert_eq!(img, golden_img);
}

#[test]
fn export_sub_raw12bpp() {
    let img = mkimage()
        .export_subsampled_image((2, 4), ImageFormat::RawLinear12BppLE)
        .unwrap();

    // std::fs::write("tests/test_sub_12bpp.raw", &img).unwrap();
    let golden_img = include_bytes!("test_sub_12bpp.raw");
    assert_eq!(img, golden_img);
}

#[test]
#[cfg(feature = "png")]
fn export_png8bpp() {
    let img = mkimage().export_image(ImageFormat::PngGamma8Bpp).unwrap();

    // std::fs::write("tests/test_8bpp.png", &img).unwrap();
    let golden_img = include_bytes!("test_8bpp.png");
    assert_eq!(img, golden_img);
}

#[test]
#[cfg(feature = "png")]
fn export_window_png8bpp() {
    let wnd = Window::new(32, 16).at(5, 8);

    let img = mkimage()
        .export_window_image(wnd, ImageFormat::PngGamma8Bpp)
        .unwrap();

    // std::fs::write("tests/test_wnd_8bpp.png", &img).unwrap();
    let golden_img = include_bytes!("test_wnd_8bpp.png");
    assert_eq!(img, golden_img);
}

#[test]
#[cfg(feature = "png")]
fn export_sub_png8bpp() {
    let img = mkimage()
        .export_subsampled_image((2, 2), ImageFormat::PngGamma8Bpp)
        .unwrap();

    // std::fs::write("tests/test_sub_8bpp.png", &img).unwrap();
    let golden_img = include_bytes!("test_sub_8bpp.png");
    assert_eq!(img, golden_img);
}

#[test]
#[cfg(feature = "png")]
fn export_png16bpp() {
    let img = mkimage().export_image(ImageFormat::PngLinear16Bpp).unwrap();

    // std::fs::write("tests/test_16bpp.png", &img).unwrap();
    let golden_img = include_bytes!("test_16bpp.png");
    assert_eq!(img, golden_img);
}

#[test]
#[cfg(feature = "png")]
fn export_window_png16bpp() {
    let wnd = Window::new(32, 16).at(5, 8);

    let img = mkimage()
        .export_window_image(wnd, ImageFormat::PngLinear16Bpp)
        .unwrap();

    // std::fs::write("tests/test_wnd_16bpp.png", &img).unwrap();
    let golden_img = include_bytes!("test_wnd_16bpp.png");
    assert_eq!(img, golden_img);
}
