use eframe::egui::{Color32, Pos2};
use std::collections::{HashMap, VecDeque as Stack};
use std::sync::{Arc, Mutex};
use std::time::Instant;

static CANVAS_WIDTH: u32 = 1500;
static CANVAS_HEIGHT: u32 = 1024;

pub fn plot_circle_pixels(x: f32, y: f32, x_c: f32, y_c: f32, pixels: &mut Vec<Pos2>) {
    let (mut sx, mut sy) = (1.0, 1.0);
    for _ in 0..4 {
        let xsx = x * sx;
        let ysy = y * sy;
        pixels.push(Pos2::new(x_c + xsx, y_c + ysy));
        pixels.push(Pos2::new(x_c + ysy, y_c + xsx));
        sx *= -1.0;
        sy *= -1.0 * sx;
    }
}

pub fn plot_ellipse_pixels(x: f32, y: f32, x_c: f32, y_c: f32, pixels: &mut Vec<Pos2>) {
    let (mut sx, mut sy) = (1.0, 1.0);
    for _ in 0..4 {
        let xsx = x * sx;
        let ysy = y * sy;
        pixels.push(Pos2::new(x_c + xsx, y_c + ysy));
        sx *= -1.0;
        sy *= -1.0 * sx;
    }
}


pub fn draw_circle(center: Pos2, radius: f32) -> Vec<Pos2> {
    let mut pixels = vec![];
    let (x_c, y_c) = (center.x, center.y);
    let sqr_radius = (radius * radius) as u32;
    let x_range = radius / std::f32::consts::SQRT_2 + 1.0;
    let mut x = 0;
    while x as f32 <= x_range {
        let y: f32 = ((sqr_radius - x * x) as f32).sqrt();
        plot_circle_pixels(x as f32, y.round(), x_c, y_c, &mut pixels);
        x += 1;
    }
    pixels
}

pub fn draw_ellipse(center: Pos2, radius: Pos2) -> Vec<Pos2> {
    let mut pixels = vec![];
    let (x_c, y_c) = (center.x, center.y);
    let sqr_a = radius.x * radius.x;
    let sqr_b = radius.y * radius.y;
    let sqrt_coeff = radius.y / radius.x;
    let hyp = (sqr_a + sqr_b).sqrt();
    let x_range = sqr_a / hyp + 1.0;
    let mut x = 0.0;
    while x <= x_range {
        let y = sqrt_coeff * (sqr_a - x * x).sqrt();
        plot_ellipse_pixels(x, y.round(), x_c, y_c, &mut pixels);
        x += 1.0;
    }

    let sqrt_coeff = sqrt_coeff.recip();
    let y_range = sqr_b / hyp + 1.0;
    let mut y = 0.0;
    while y <= y_range {
        let x = sqrt_coeff * (sqr_b - y * y).sqrt();
        plot_ellipse_pixels(x.round(), y, x_c, y_c, &mut pixels);
        y += 1.0;
    }

    pixels
}


pub fn dda(points: [Pos2; 2]) -> Vec<Pos2> {
    if points[0] == points[1] {
        return [points[0]].into();
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

#[derive(Debug)]
pub struct Canvas {
    pub strings: Vec<((Pos2, Pos2), Color32)>,
    points: Vec<(Pos2, Color32)>,
    pub circles: Vec<(Pos2, f32, Color32)>,
    pub ellipse: Vec<(Pos2, Pos2, Color32)>,
    pixels_fill: HashMap<(u32, u32), (u8, u8, u8)>,
    pub pixels_edges: HashMap<(u32, u32), (u8, u8, u8)>,
    close: Vec<usize>,
    pub background: Color32,

    pub bebra: Vec<(Pos2, Color32)>,
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            strings: vec![],
            points: vec![],
            circles: vec![],
            ellipse: vec![],
            pixels_fill: Default::default(),
            pixels_edges: Default::default(),
            bebra: vec![],
            close: vec![0],
            background: Color32::WHITE,
        }
    }

    pub fn last_closed_point(&self) -> Option<&Pos2> {
        let len = self.close.last()?;
        if self.points.len() > len.to_owned() {
            self.last_point()
        } else {
            None
        }
    }

    pub fn last_point(&self) -> Option<&Pos2> {
        let point = self.points.last()?;
        Some(&point.0)
    }

    pub fn points(&self) -> &[(Pos2, Color32)] {
        &self.points
    }

    pub fn closes(&self) -> &[usize] {
        &self.close
    }

    pub fn add_point(&mut self, pos2: Pos2, color32: Color32) {
        if self.points.len() - self.close.last().unwrap() >= 1 {
            let pos1 = self.points[self.points.len() - 1].0;
            let pos2 = pos2;
            for (&i, c) in dda([pos1, pos2]).iter().map(|x| (x, color32)) {
                let (r, g, b, _) = c.to_tuple();
                self.pixels_edges.insert((i.x as u32, i.y as u32), (r, g, b));
                self.bebra.push((i, c));
            }
        }
        self.points.push((pos2, color32));
    }

    pub fn add_circle(&mut self, pos2: Pos2, r: f32, color32: Color32) {
        for (&i, c) in draw_circle(pos2, r).iter().map(|x| (x, color32)) {
            let (r, g, b, _) = c.to_tuple();
            self.pixels_edges.insert((i.x as u32, i.y as u32), (r, g, b));
            self.bebra.push((i, c));
        }
        self.circles.push((pos2, r, color32));
    }

    pub fn add_ellipse(&mut self, pos1: Pos2, pos2: Pos2, color32: Color32) {
        for (&i, c) in draw_ellipse(pos1, pos2).iter().map(|x| (x, color32)) {
            let (r, g, b, _) = c.to_tuple();
            self.pixels_edges.insert((i.x as u32, i.y as u32), (r, g, b));
            self.bebra.push((i, c));
        }
        self.ellipse.push((pos1, pos2, color32));
    }

    pub fn clear(&mut self) {
        self.points.clear();
        self.close.clear();
        self.close.push(0);
        self.strings.clear();
        self.bebra.clear();
        self.pixels_fill.clear();
        self.pixels_edges.clear();
        self.circles.clear();
        self.ellipse.clear();
    }

    pub fn clean(&mut self) {
        self.strings.clear();
        self.pixels_fill.clear();
    }

    pub fn close(&mut self) -> Option<()> {
        let &index = self.close.last()?;
        if self.points.len() - index >= 3 {
            let (pos, color) = self.points[index];
            self.add_point(pos, color);
            self.close.push(self.points.len());
            Some(())
        } else {
            None
        }
    }

    pub fn at(&self, x: u32, y: u32) -> Color32 {
        let &(r, g, b) = self.pixels_fill.get(&(x, y)).unwrap_or({
            self.pixels_edges.get(&(x, y)).unwrap_or({
                let (r, g, b, _) = self.background.to_tuple();
                &(r, g, b)
            })
        });
        Color32::from_rgb(r, g, b)
    }

    pub fn draw_line(&mut self, pos1: (u32, u32), pos2: (u32, u32), color32: Color32) {
        let (r, g, b, _) = color32.to_tuple();
        for i in pos1.0..=pos2.0 {
            self.pixels_fill.insert((i, pos2.1), (r, g, b));
        }
        self.strings.push((
            (
                Pos2::new(pos1.0 as f32 - 1., pos1.1 as f32),
                Pos2::new(pos2.0 as f32 + 1., pos2.1 as f32),
            ),
            color32,
        ))
    }

    pub fn eq_color(&self, x: u32, y: u32, color32: Color32) -> bool {
        self.at(x, y) == color32
    }


    pub fn filling(
        seed: Pos2,
        canvas: &Arc<Mutex<Self>>,
        fill: Color32,
        border: Color32,
        dur: &mut Arc<Mutex<std::time::Duration>>,
        delay: u64,
        rec: bool,
    ) {
        let (x, y) = (seed.x as u32, seed.y as u32);
        if rec {
            let start = std::time::Instant::now();
            fill_recursive(x, y, canvas, fill, border, dur, delay, start);
        } else {
            filling_ordinary(x, y, canvas, fill, border, dur, delay);
        }
    }
}

pub fn filling_ordinary(
    x: u32, y: u32,
    canvas: &Arc<Mutex<Canvas>>,
    fill: Color32,
    border: Color32,
    dur: &mut Arc<Mutex<std::time::Duration>>,
    delay: u64,
) {
    let start = std::time::Instant::now();
    let mut stack: Stack<(u32, u32)> = Stack::new();
    stack.push_back((x, y));
    while let Some(cur_pixel) = stack.pop_back() {
        let (x, y) = (cur_pixel.0, cur_pixel.1);
        let right_x = (x + 1..=CANVAS_WIDTH)
            .take_while(|&tmp_x| {
                !canvas.lock().unwrap().eq_color(tmp_x, y, border)
                    && !canvas.lock().unwrap().eq_color(tmp_x, y, fill)
            })
            .last()
            .unwrap_or(x);

        let left_x = (0..=x)
            .rev()
            .take_while(|&tmp_x| {
                !canvas.lock().unwrap().eq_color(tmp_x, y, border)
                    && !canvas.lock().unwrap().eq_color(tmp_x, y, fill)
            })
            .last()
            .map_or(x, |tmp_x| tmp_x);

        canvas.lock().unwrap().draw_line((left_x, y), (right_x, y), fill);
        std::thread::sleep(std::time::Duration::from_millis(delay));
        *dur.lock().unwrap() = start.elapsed();
        for y in [y + 1, y - 1] {
            let mut tmp_x = left_x;
            while tmp_x <= right_x {
                let mut flag = false;
                while tmp_x <= right_x && !canvas.lock().unwrap().eq_color(tmp_x, y, border) && !canvas.lock().unwrap().eq_color(tmp_x, y, fill) {
                    flag = true;
                    tmp_x += 1;
                }
                if flag && y < CANVAS_HEIGHT - 1 && y > 0 {
                    let x = if tmp_x <= right_x && !canvas.lock().unwrap().eq_color(tmp_x, y, border) && !canvas.lock().unwrap().eq_color(tmp_x, y, fill) {
                        tmp_x
                    } else {
                        tmp_x - 1
                    };
                    stack.push_back((x, y));
                    flag = false;
                }
                let begin_x = tmp_x;
                while tmp_x <= right_x && (canvas.lock().unwrap().eq_color(tmp_x, y, border) || canvas.lock().unwrap().eq_color(tmp_x, y, fill)) {
                    tmp_x += 1;
                }
                if tmp_x == begin_x {
                    tmp_x += 1;
                }
            }
        }
    }
}

pub fn fill_recursive(x: u32, y: u32, canvas: &Arc<Mutex<Canvas>>, fill: Color32, border: Color32, dur: &mut Arc<Mutex<std::time::Duration>>,
                      delay: u64, start: Instant) {
    if canvas.lock().unwrap().eq_color(x, y, border) || canvas.lock().unwrap().eq_color(x, y, fill) {
        return;
    }

    let right_x = (x + 1..=CANVAS_WIDTH)
        .take_while(|&tmp_x| !canvas.lock().unwrap().eq_color(tmp_x, y, border) && !canvas.lock().unwrap().eq_color(tmp_x, y, fill))
        .last()
        .unwrap_or(x);

    let left_x = (0..=x)
        .rev()
        .take_while(|&tmp_x| !canvas.lock().unwrap().eq_color(tmp_x, y, border) && !canvas.lock().unwrap().eq_color(tmp_x, y, fill))
        .last()
        .map_or(x, |tmp_x| tmp_x + 1);

    canvas.lock().unwrap().draw_line((left_x, y), (right_x, y), fill);
    std::thread::sleep(std::time::Duration::from_millis(delay));
    *dur.lock().unwrap() = start.elapsed();
    for y in [y - 1, y + 1] {
        let mut tmp_x = left_x;
        while tmp_x <= right_x {
            let mut flag = false;
            while tmp_x <= right_x && !canvas.lock().unwrap().eq_color(tmp_x, y, border) && !canvas.lock().unwrap().eq_color(tmp_x, y, fill) {
                flag = true;
                tmp_x += 1;
            }
            if flag && y < CANVAS_HEIGHT - 1 && y > 0 {
                let x = if tmp_x <= right_x && !canvas.lock().unwrap().eq_color(tmp_x, y, border) && !canvas.lock().unwrap().eq_color(tmp_x, y, fill) {
                    tmp_x
                } else {
                    tmp_x - 1
                };
                fill_recursive(x, y, canvas, fill, border, dur, delay, start);
            }
            let begin_x = tmp_x;
            while tmp_x <= right_x && (canvas.lock().unwrap().eq_color(tmp_x, y, border) || canvas.lock().unwrap().eq_color(tmp_x, y, fill)) {
                tmp_x += 1;
            }
            if tmp_x == begin_x {
                tmp_x += 1;
            }
        }
    }
}
