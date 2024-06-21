use eframe::egui::{pos2, Color32, Pos2};
use rand::seq::SliceRandom;
use rand::Rng;

pub fn rotate(point: &mut Pos2, angle: f32, center: &Pos2) {
    let x = point[0];
    let y = point[1];

    let translated_x = x - center[0];
    let translated_y = y - center[1];

    let angle = angle.to_radians();
    let angle_cos = angle.cos();
    let angle_sin = angle.sin();
    let rotated_x = translated_x * angle_cos - translated_y * angle_sin;
    let rotated_y = translated_x * angle_sin + translated_y * angle_cos;

    point[0] = rotated_x + center[0];
    point[1] = rotated_y + center[1];
}

pub fn abate_color(pixel: Color32, canvas: Color32, i: f32) -> Color32 {
    let (r1, g1, b1, _) = pixel.to_tuple();
    let (r2, g2, b2, _) = canvas.to_tuple();
    let (r_ratio, b_ratio, g_ratio) = (
        r2 as f32 - r1 as f32,
        g2 as f32 - g1 as f32,
        b2 as f32 - b1 as f32,
    );
    let (r_ratio, b_ratio, g_ratio) = (r_ratio / 255.0, g_ratio / 255.0, b_ratio / 255.0);
    let i = if i >= 255.0 { 255 } else { i as u8 };
    Color32::from_rgb(
        r1 + (r_ratio * i as f32) as u8,
        g1 + (g_ratio * i as f32) as u8,
        b1 + (b_ratio * i as f32) as u8,
    )
}

pub fn measure_time<F, T>(mut f: F, len: f32) -> u128
where
    F: FnMut(&[Pos2; 2]) -> T,
{
    let mut rng = rand::thread_rng();
    let point1 = pos2(rng.gen_range(0..1000) as f32, rng.gen_range(0..1000) as f32);
    let n = 5000;
    let points = (1..n)
        .map(|_| {
            let mut point2 = point1 + [len as f32, 0.0].into();
            rotate(&mut point2, rng.gen_range(0..360) as f32, &point1);
            point2
        })
        .collect::<Vec<Pos2>>();
    let start = std::time::Instant::now();
    for _ in 1..n {
        f(&[point1, points.choose(&mut rng).unwrap().clone()]);
    }
    start.elapsed().as_nanos() / n as u128
}

pub fn count_steps(points: &[Pos2]) -> i32 {
    let mut n = 0;
    let (mut x, mut y) = points[0].into();
    for i in 0..points.len() {
        let (x_new, y_new) = points[i].into();
        if x_new != x && y_new != y {
            n += 1;
        }
        (x, y) = (x_new, y_new);
    }
    n
}

pub fn count_steps_wu(points: &[Pos2]) -> i32 {
    let mut n = 0;
    let (mut x, mut y) = points[0].into();
    for i in 0..points.len() / 2 {
        let (x_new, y_new) = points[i * 2].into();
        if x_new != x && y_new != y {
            n += 1;
        }
        (x, y) = (x_new, y_new);
    }
    n
}

pub fn measure_jaggies<F>(mut f: F, len: f32) -> Vec<(i32, i32)>
where
    F: FnMut(&[Pos2; 2]) -> Vec<Pos2>,
{
    let point1 = Pos2::ZERO;
    let mut res = vec![];
    for i in 0..=90 {
        let mut point2 = point1 + [len as f32, 0.0].into();
        rotate(&mut point2, i as f32, &point1);
        let cs = f(&[point1, point2]);
        res.push((i, count_steps(&cs)));
    }
    res
}

pub fn measure_jaggies_bj<F>(mut f: F, len: f32) -> Vec<(i32, i32)>
where
    F: FnMut(&[Pos2; 2]) -> Vec<(Pos2, f32)>,
{
    let point1 = Pos2::ZERO;
    let mut res = vec![];
    for i in 0..=90 {
        let mut point2 = point1 + [len as f32, 0.0].into();
        rotate(&mut point2, i as f32, &point1);
        let cs = f(&[point1, point2]).iter().map(|x| x.0).collect::<Vec<_>>();
        res.push((i, count_steps(&cs)));
    }
    res
}

pub fn measure_jaggies_wu<F>(mut f: F, len: f32) -> Vec<(i32, i32)>
where
    F: FnMut(&[Pos2; 2]) -> Vec<(Pos2, f32)>,
{
    let point1 = Pos2::ZERO;
    let mut res = vec![];
    for i in 0..=90 {
        let mut point2 = point1 + [len as f32, 0.0].into();
        rotate(&mut point2, i as f32, &point1);
        let cs = f(&[point1, point2]).iter().map(|x| x.0).collect::<Vec<_>>();
        res.push((i, count_steps_wu(&cs)));
    }
    res
}
