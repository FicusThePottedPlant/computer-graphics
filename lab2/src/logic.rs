pub fn find_root<F>(mut f: F, mut a: f64, mut b: f64, epsilon: f64) -> Option<f64>
where
    F: FnMut(f64) -> f64,
{
    while (b - a).abs() > epsilon {
        let c = (a + b) / 2.0;
        if f(c).abs() < epsilon {
            return Some(c);
        }
        if f(a) * f(c) < 0.0 {
            b = c;
        } else {
            a = c;
        }
    }
    Some((a + b) / 2.0)
}

pub fn transform(point: &mut [f64; 2], dx: f64, dy: f64) {
    point[0] += dx;
    point[1] += dy;
}

pub fn scale(point: &mut [f64; 2], kx: f64, ky: f64, cx: f64, cy: f64) {
    let x = point[0];
    let y = point[1];
    point[0] = (x - cx) * kx + cx;
    point[1] = (y - cy) * ky + cy;
}

pub fn rotate(point: &mut [f64; 2], angle: f64, cx: f64, cy: f64) {
    let x = point[0];
    let y = point[1];

    let translated_x = x - cx;
    let translated_y = y - cy;

    let angle = angle.to_radians();
    let angle_cos = angle.cos();
    let angle_sin = angle.sin();
    let rotated_x = translated_x * angle_cos - translated_y * angle_sin;
    let rotated_y = translated_x * angle_sin + translated_y * angle_cos;

    point[0] = rotated_x + cx;
    point[1] = rotated_y + cy;
}
