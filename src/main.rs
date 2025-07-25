use minifb::{Window, WindowOptions};
use rand::Rng;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

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

    let n = 10;
    let points = init_citiy_points(n, (width / 2) - 20, height);
    let rect_width: u32 = 10;
    let rect_height: u32 = 10;
    let half_r_width = (rect_width / 2) as i32;
    let half_r_height = (rect_height / 2) as i32;

    let mut frame_count = 0;

    let draw_point = |(x, y): (u32, u32)| {
        let mut buffer_ref = buffer.borrow_mut();
        for i in y..y + rect_height {
            let start_idx = (i * width as u32 + x) as usize;
            let end_idx =
                (start_idx as u32 + rect_width).min(i * width as u32 + width as u32) as usize;
            if start_idx < end_idx {
                buffer_ref[start_idx..end_idx].fill(0x000000);
            }
        }
    };

    let connect_points = |(i_x, i_y): (u32, u32), (j_x, j_y): (u32, u32)| {
        // Make the coordinates the center of the dots
        let mut x0 = i_x as i32 + half_r_width;
        let mut y0 = i_y as i32 + half_r_height;

        let x1 = j_x as i32 + half_r_width;
        let y1 = j_y as i32 + half_r_height;

        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -1 * (y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut error = dx + dy;

        let mut buffer_ref = buffer.borrow_mut();

        loop {
            let temp = y0 as usize * width + x0 as usize;
            buffer_ref[temp - 1..temp + 1].fill(0x00FF00); // Make the line thicker
            let e2 = 2 * error;
            if e2 >= dy {
                if x0 == x1 {
                    break;
                };
                error = error + dy;
                x0 = x0 + sx;
            }
            if e2 <= dx {
                if y0 == y1 {
                    break;
                };
                error = error + dx;
                y0 = y0 + sy
            }
        }
    };

    while window.is_open() {
        let start = Instant::now();

        buffer.borrow_mut().fill(0xFFFFFF);

        for (i, cords) in &points {
            for (j, other_cords) in &points {
                if i != j {
                    connect_points(*cords, *other_cords);
                }
            }
        }

        for (_, point) in &points {
            draw_point(*point);
        }

        window
            .update_with_buffer(&buffer.borrow(), width, height)
            .unwrap();

        frame_count += 1;
        if frame_count % 100 == 0 {
            let duration = start.elapsed();
            println!("Frame time: {:.2?}", duration);
        }
    }
}

fn make_key(i: u16, j: u16) -> (u16, u16) {
    if i < j {
        (i, j)
    } else {
        (j, i)
    }
}

fn distance((i_x, i_y): (u32, u32), (j_x, j_y): (u32, u32)) -> u32 {
    (i_x - j_x) * (i_x - j_x) + (i_y - j_y) * (i_y - j_y)
}

fn init_citiy_points(n: u16, width: usize, height: usize) -> HashMap<u16, (u32, u32)> {
    let mut rng = rand::thread_rng();
    let mut map: HashMap<u16, (u32, u32)> = HashMap::new();
    let mut existing: Vec<(u32, u32)> = Vec::new();

    for i in 0..n {
        let mut x: u32 = rng.gen_range(0..width as u32);
        let mut y: u32 = rng.gen_range(0..height as u32);

        // Make sure that this point doesn't exist
        while existing.contains(&(x, y)) {
            x = rng.gen_range(0..width as u32);
            y = rng.gen_range(0..height as u32);
        }

        existing.push((x, y));

        map.insert(i, (x, y));
    }

    map
}

fn init_distance_matrix(map: HashMap<u16, (u32, u32)>) -> HashMap<(u16, u16), u32> {
    let mut distances: HashMap<(u16, u16), u32> = HashMap::new();

    for (i, i_cords) in &map {
        for (j, j_cords) in &map {
            if i != j {
                distances.insert(make_key(*i, *j), distance(*i_cords, *j_cords));
            }
        }
    }

    distances
}

fn held_karp(matrix: HashMap<(u16, u16), u32>, n: u16) -> (u32, Vec<u16>) {
    let size = 1 << n;
    let mut dp: Vec<Vec<u32>> = vec![vec![u32::MAX; n as usize]; size];
    let mut parent: Vec<Vec<Option<u16>>> = vec![vec![None; n as usize]; size];

    dp[1][0] = 0; // Start at city 0

    for subset in 1..size {
        if subset & 1 == 0 {
            continue; // Only consider subsets containing the start city
        }
        for last in 0..n {
            if (subset & (1 << last)) == 0 {
                continue; // last not in subset
            }
            if last == 0 && subset != 1 {
                continue; // Only allow start city as the first city
            }
            let prev_subset = subset ^ (1 << last);
            if prev_subset == 0 && last == 0 {
                continue; // skip the trivial case
            }
            for prev in 0..n {
                if prev == last || (prev_subset & (1 << prev)) == 0 {
                    continue;
                }
                let key = if last < prev {
                    (last, prev)
                } else {
                    (prev, last)
                };
                let cost = dp[prev_subset as usize][prev as usize]
                    .saturating_add(*matrix.get(&key).unwrap_or(&u32::MAX));
                if cost < dp[subset as usize][last as usize] {
                    dp[subset as usize][last as usize] = cost;
                    parent[subset as usize][last as usize] = Some(prev);
                }
            }
        }
    }

    // Find the minimum cost to return to the start city
    let mut min_cost = u32::MAX;
    let mut last_city = 0;
    for i in 1..n {
        let key = (0, i);
        let cost = dp[(size - 1) as usize][i as usize]
            .saturating_add(*matrix.get(&key).unwrap_or(&u32::MAX));
        if cost < min_cost {
            min_cost = cost;
            last_city = i;
        }
    }

    // Reconstruct path
    let mut path = Vec::with_capacity(n as usize + 1);
    let mut subset = size - 1;
    let mut city = last_city;
    for _ in 0..n {
        path.push(city);
        if let Some(prev) = parent[subset as usize][city as usize] {
            subset ^= 1 << city;
            city = prev;
        } else {
            break;
        }
    }
    path.push(0); // start city
    path.reverse();

    (min_cost, path)
}
