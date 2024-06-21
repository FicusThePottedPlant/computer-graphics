use eframe::egui::{pos2, Pos2};
use std::mem::swap;

#[derive(Default, Debug, PartialEq)]
pub(crate) enum Algo {
    #[default]
    DDA,
    BresenhamFloat,
    BresenhamReal,
    BresenhamJaggiesLess,
    WU,
    BuiltIn,
}

pub fn dda(points: &[Pos2; 2]) -> Vec<Pos2> {
    if points[0] == points[1] {
        return [(points[0])].into();
    }
    let mut res = Vec::new();
    let (x1, y1) = (points[0].x, points[0].y);
    let (x2, y2) = (points[1].x, points[1].y);

    let dx = x2 - x1;
    let dy = y2 - y1;
    let abs_x = dx.abs();
    let abs_y = dy.abs();
    let l = if abs_x >= abs_y { abs_x } else { abs_y };

    let dx = dx / l;
    let dy = dy / l;

    let mut x = x1;
    let mut y = y1;
    for _ in 0..l as i32 {
        res.push(Pos2::new(x.round(), y.round()));
        x += dx;
        y += dy;
    }
    res
}

pub fn bresenham_float(points: &[Pos2; 2]) -> Vec<Pos2> {
    if points[0] == points[1] {
        return [points[0]].into();
    }
    let mut res: Vec<Pos2> = vec![];
    let (x1, y1) = (points[0].x, points[0].y);
    let (x2, y2) = (points[1].x, points[1].y);

    let mut x = x1;
    let mut y = y1;
    let dx = x2 - x1;
    let dy = y2 - y1;
    let sx = dx.signum();
    let sy = dy.signum();

    let mut dx = dx.abs();
    let mut dy = dy.abs();

    let swapped = if dx > dy {
        false
    } else {
        swap(&mut dx, &mut dy);
        true
    };

    let m = dy / dx;
    let mut e = m - 0.5;
    for _ in 0..dx as i32 {
        res.push(Pos2::new(x, y));
        if !e.is_sign_negative() {
            if swapped {
                x = x + sx;
            } else {
                y = y + sy;
            }
            e = e - 1.0;
        }
        if swapped {
            y = y + sy;
        } else {
            x = x + sx;
        }
        e = e + m;
    }
    res
}

pub fn bresenham_jaggiesless(points: &[Pos2; 2]) -> Vec<(Pos2, f32)> {
    if points[0] == points[1] {
        return [(points[0], 0.0)].into();
    }
    let mut res: Vec<(Pos2, f32)> = vec![];
    let (x1, y1) = (points[0].x, points[0].y);
    let (x2, y2) = (points[1].x, points[1].y);

    let mut x = x1;
    let mut y = y1;
    let dx = x2 - x1;
    let dy = y2 - y1;
    let sx = dx.signum();
    let sy = dy.signum();

    let mut dx = dx.abs();
    let mut dy = dy.abs();

    let swapped = if dx > dy {
        false
    } else {
        swap(&mut dx, &mut dy);
        true
    };
    let intense = 255.0;
    let m = (intense * dy) / dx;
    let w = intense - m;
    let mut e = 0.5 * intense;
    for _ in 0..dx as i32 {
        res.push((Pos2::new(x, y), e));
        if e < w {
            if !swapped {
                x = x + sx;
            } else {
                y = y + sy;
            }
            e = e + m;
        } else {
            x = x + sx;
            y = y + sy;
            e = e - w;
        }
    }
    res
}

pub fn bresenham_int(points: &[Pos2; 2]) -> Vec<Pos2> {
    if points[0] == points[1] {
        return [points[0]].into();
    }
    let mut res: Vec<Pos2> = vec![];
    let (x1, y1) = (points[0].x, points[0].y);
    let (x2, y2) = (points[1].x, points[1].y);

    let mut x = x1;
    let mut y = y1;
    let dx = x2 - x1;
    let dy = y2 - y1;
    let sx = dx.signum();
    let sy = dy.signum();

    let mut dx = dx.abs();
    let mut dy = dy.abs();

    let swapped = if dx > dy {
        false
    } else {
        swap(&mut dx, &mut dy);
        true
    };

    let mut e = 2.0 * dy - dx;
    for _ in 0..dx as i32 {
        res.push(Pos2::new(x, y));
        if !e.is_sign_negative() {
            if swapped {
                x = x + sx;
            } else {
                y = y + sy;
            }
            e = e - 2.0 * dx;
        }
        if swapped {
            y = y + sy;
        } else {
            x = x + sx;
        }
        e = e + 2.0 * dy;
    }
    res
}

pub fn wu(points: &[Pos2; 2]) -> Vec<(Pos2, f32)> {
    if points[0] == points[1] {
        return [(points[0], 0.0)].into();
    }
    let mut pixels: Vec<(Pos2, f32)> = Vec::new();
    let mut x1 = points[0].x;
    let mut y1 = points[0].y;
    let mut x2 = points[1].x;
    let mut y2 = points[1].y;

    let steep = (y2 - y1).abs() > (x2 - x1).abs();
    if steep {
        swap(&mut x1, &mut y1);
        swap(&mut x2, &mut y2);
    }
    if x2 < x1 {
        swap(&mut x1, &mut x2);
        swap(&mut y1, &mut y2);
    }
    let dx = x2 - x1;
    let dy = y2 - y1;
    let grad = if dx == 0.0 {
        1.0
    } else {
        dy as f32 / dx as f32
    };

    if steep {
        pixels.push((pos2(y1, x1 as f32), M_I));
    } else {
        pixels.push((pos2(x1 as f32, y1), M_I));
    }

    if steep {
        pixels.push((pos2(y2, x2 as f32), M_I));
    } else {
        pixels.push((pos2(x2 as f32, y2), M_I));
    }

    static M_I: f32 = 255.0;
    let xpxl1 = x1;
    let xpxl2 = x2;

    let mut intery = y1 + grad;
    let (xpxl1, xpxl2) = (xpxl1 as i32, xpxl2 as i32);
    for x in xpxl1 + 1..=xpxl2 - 1 {
        let y = intery.trunc();
        let fpart = intery.fract();
        if steep {
            pixels.push((pos2(y, x as f32), M_I * fpart));
            pixels.push((pos2(y + 1.0, x as f32), M_I * (1.0 - fpart)));
        } else {
            pixels.push((pos2(x as f32, y), M_I * fpart));
            pixels.push((pos2(x as f32, y + 1.0), M_I * (1.0 - fpart)));
        }
        intery += grad;
    }

    pixels
}
