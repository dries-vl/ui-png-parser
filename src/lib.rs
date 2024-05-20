use std::collections::VecDeque;
use std::ffi::{c_char, CStr};
use image::{GenericImageView, ImageBuffer, Rgba};

#[repr(C)]
pub struct Region {
    pub color: [u8; 4],
    pub bounds: (u32, u32, u32, u32),
}

#[no_mangle]
pub extern "C" fn parse_image(path: *const c_char, out_count: *mut usize) -> *const Region {

    let mut regions = vec![];

    let c_str = unsafe { CStr::from_ptr(path) };
    let r_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return regions.as_ptr(),
    };

    let img = image::open(r_str).expect("Failed to open image");

    // Analyze the image
    let (image_width, image_height) = img.dimensions();
    let mut processed = ImageBuffer::from_pixel(image_width, image_height, Rgba([0, 0, 0, 0]));

    for x in 0..image_width {
        for y in 0..image_height {
            let pixel = img.get_pixel(x, y);
            // Ignore white, black, and already processed pixels
            if pixel[0] == 255 && pixel[1] == 255 && pixel[2] == 255
                || pixel[0] == 0 && pixel[1] == 0 && pixel[2] == 0
                || processed.get_pixel(x, y)[3] != 0 {
                continue;
            }

            // Perform flood fill to find bounds
            let bounds = flood_fill(&img, &mut processed, x, y, pixel);
            regions.push(Region {color: [pixel[0], pixel[1], pixel[2], pixel[3]], bounds});
        }
    }

    unsafe { *out_count = regions.len(); }
    let boxed_regions = regions.into_boxed_slice();
    let ptr = boxed_regions.as_ptr();
    std::mem::forget(boxed_regions); // Prevent Rust from cleaning up the memory
    ptr
}


fn flood_fill(
    img: &image::DynamicImage,
    processed: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    start_x: u32,
    start_y: u32,
    target_color: Rgba<u8>,
) -> (u32, u32, u32, u32) {
    let (width, height) = img.dimensions();
    let mut queue = VecDeque::new();
    queue.push_back((start_x, start_y));

    let mut min_x = start_x;
    let mut max_x = start_x;
    let mut min_y = start_y;
    let mut max_y = start_y;

    while let Some((x, y)) = queue.pop_front() {
        if processed.get_pixel(x, y)[3] != 0 {
            continue;
        }

        // Mark this pixel as processed
        processed.put_pixel(x, y, Rgba([255, 255, 255, 255]));

        // Update bounds
        if x < min_x { min_x = x; }
        if x > max_x { max_x = x; }
        if y < min_y { min_y = y; }
        if y > max_y { max_y = y; }

        // Check neighbors
        for &(dx, dy) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                let neighbor = img.get_pixel(nx as u32, ny as u32);
                if neighbor == target_color && processed.get_pixel(nx as u32, ny as u32)[3] == 0 {
                    queue.push_back((nx as u32, ny as u32));
                }
            }
        }
    }

    (min_x, min_y, max_x, max_y)
}
