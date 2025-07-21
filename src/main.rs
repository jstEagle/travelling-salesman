use minifb::{Window, WindowOptions};
use std::time::Instant;
use std::rc::Rc;
use std::cell::RefCell;

fn main() {
    let width = 900;
    let height = 600;
    let buffer = Rc::new(RefCell::new(vec![0xFFFFFF; width * height]));

    let mut window = Window::new(
        "Travelling Salesman Visualization",
        width,
        height,
        WindowOptions {
            resize: false,
            ..WindowOptions::default()
        },
    ).unwrap();

    let points: Vec<(u32, u32)> = vec![(100, 100), (150, 200), (200, 100), (160, 30)];
    let rect_width: u32= 10;
    let rect_height: u32 = 10;
    let half_r_width = (rect_width / 2) as i32;
    let half_r_height = (rect_height / 2) as i32;

    let mut frame_count = 0;


    let draw_point = |(x, y): (u32, u32)| {
        let mut buffer_ref = buffer.borrow_mut();
        for i in y..y + rect_height {
            let start_idx = (i * width as u32 + x) as usize;
            let end_idx = (start_idx as u32 + rect_width).min(i * width as u32 + width as u32) as usize;
            if start_idx < end_idx {
                buffer_ref[start_idx..end_idx].fill(0x000000);
            }
        }
    };

    let connect_points = |(x_first, y_first): (u32, u32), (x_second, y_second): (u32, u32)| {
        // Make the coordinates the center of the dots
        let mut x0 = x_first as i32 + half_r_width;
        let mut y0 = y_first as i32 + half_r_height;

        let x1 = x_second as i32 + half_r_width;
        let y1 = y_second as i32 + half_r_height;

        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -1 * (y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut error = dx + dy;
        
        let mut buffer_ref = buffer.borrow_mut();

        loop {
            let temp = y0 as usize * width + x0 as usize;
            buffer_ref[temp-1..temp+1].fill(0x00FF00); // Make the line thicker
            let e2 = 2 * error;
            if e2 >= dy {
                if x0 == x1 { break };
                error = error + dy;
                x0 = x0 + sx;
            }
            if e2 <= dx {
                if y0 == y1 { break };
                error = error + dx;
                y0 = y0 + sy
            }
        }
    };

    while window.is_open() {
        let start = Instant::now();

                buffer.borrow_mut().fill(0xFFFFFF);

        for point in &points {
            for other in &points {
                connect_points(*point, *other);
            }
        }

        for point in &points {
            draw_point(*point);
        }


        window.update_with_buffer(&buffer.borrow(), width, height).unwrap();

        frame_count += 1;
        if frame_count % 100 == 0 {
            let duration = start.elapsed();
            println!("Frame time: {:.2?}", duration);
        }
    }
}