use minifb::{Window, WindowOptions};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

mod held_karp;
mod helpers;
use held_karp::held_karp;
use helpers::{init_citiy_points, init_distance_matrix};

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
    )
    .unwrap();

    let n = 15;
    let points = init_citiy_points(n, width as usize, height);
    let rect_width: u32 = 10;
    let rect_height: u32 = 10;
    let half_r_width = (rect_width / 2) as i32;
    let half_r_height = (rect_height / 2) as i32;

    let draw_point = |(x, y): (u32, u32)| {
        let mut buffer_ref = buffer.borrow_mut();
        let max_x = (buffer_ref.len() / width) as u32;
        let max_y = width as u32;
        let x0 = x
            .saturating_sub(half_r_width as u32)
            .min(max_y.saturating_sub(rect_width));
        let y0 = y
            .saturating_sub(half_r_height as u32)
            .min(max_x.saturating_sub(rect_height));
        for i in y0..(y0 + rect_height).min(max_x) {
            let start_idx = (i * width as u32 + x0) as usize;
            let end_idx =
                (start_idx as u32 + rect_width).min(i * width as u32 + width as u32) as usize;
            if start_idx < end_idx && end_idx <= buffer_ref.len() {
                buffer_ref[start_idx..end_idx].fill(0x000000);
            }
        }
    };

    let connect_points = |(i_x, i_y): (u32, u32), (j_x, j_y): (u32, u32), color: u32| {
        let mut x0 = i_x as i32;
        let mut y0 = i_y as i32;
        let x1 = j_x as i32;
        let y1 = j_y as i32;
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -1 * (y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut error = dx + dy;
        let mut buffer_ref = buffer.borrow_mut();
        loop {
            if x0 >= 0 && y0 >= 0 && (x0 as usize) < width && (y0 as usize) < height {
                let temp = y0 as usize * width + x0 as usize;
                if temp > 0 && temp + 1 < buffer_ref.len() {
                    buffer_ref[temp - 1..temp + 1].fill(color);
                }
            }
            let e2 = 2 * error;
            if e2 >= dy {
                if x0 == x1 {
                    break;
                }
                error = error + dy;
                x0 = x0 + sx;
            }
            if e2 <= dx {
                if y0 == y1 {
                    break;
                }
                error = error + dx;
                y0 = y0 + sy
            }
        }
    };

    // Draw all points once at the beginning
    for (_, point) in &points {
        draw_point(*point);
    }
    window
        .update_with_buffer(&buffer.borrow(), width, height)
        .unwrap();

    // Run Held-Karp to completion in one go
    let matrix = init_distance_matrix(points.clone());
    let start = Instant::now();
    let (best_path, min_cost) = held_karp(matrix, n);
    let duration = start.elapsed();
    println!("Held-Karp completed in: {:.2?}", duration);
    println!("Best cost: {}", min_cost);

    // Draw best path as final solution
    buffer.borrow_mut().fill(0xFFFFFF);
    // Then draw the best path on top
    if best_path.len() > 1 {
        for w in best_path.windows(2) {
            let i = w[0];
            let j = w[1];
            connect_points(points[&i], points[&j], 0x32a0a8);
        }
    }
    // Draw all points first
    for (_, point) in &points {
        draw_point(*point);
    }
    window
        .update_with_buffer(&buffer.borrow(), width, height)
        .unwrap();
    while window.is_open() {
        window.update();
    }
}
