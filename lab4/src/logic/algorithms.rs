#![allow(arithmetic_overflow)]


use std::fs::OpenOptions;
use eframe::egui;
use eframe::egui::{Pos2, Vec2};
use std::io::{Write};

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub enum DrawType {
    #[default]
    CANONICAL,
    PARAMETRIC,
    MIDPOINT,
    BRESENHAM,
    BuiltIn,
}

pub trait Measurable {
    fn measure_time(&mut self, draw_type: DrawType, rad: f32, file: &std::fs::File);
}

pub struct Circle {
    center: Pos2,
    radius: f32,
    profile: bool
}

impl Circle {
    pub fn new(center: Pos2, radius: f32) -> Self {
        Circle { center, radius, profile: false }
    }

    fn plot_circle_pixels(&self, x: f32, y: f32, x_c: f32, y_c: f32, pixels: &mut Vec<Pos2>) {
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

    pub fn draw_canonic(&self) -> Vec<Pos2> {
        let mut pixels = vec![];
        let (x_c, y_c) = (self.center.x, self.center.y);
        let sqr_radius = (self.radius * self.radius) as u32;
        let x_range = self.radius / std::f32::consts::SQRT_2 + 1.0;
        let mut x = 0;
        while x as f32 <= x_range {
            let y: f32 = ((sqr_radius - x * x) as f32).sqrt();
            if !self.profile {
                self.plot_circle_pixels(x as f32, y.round(), x_c, y_c, &mut pixels);
            }
            x += 1;
        }
        pixels
    }

    pub fn draw_parametric(&self) -> Vec<Pos2> {
        let mut pixels = vec![];
        let (x_c, y_c) = (self.center.x, self.center.y);
        let t_range = std::f32::consts::FRAC_PI_4;
        let t_step = self.radius.recip();
        let mut t = 0.0;
        while t <= t_range {
            let x = self.radius * t.cos();
            let y = self.radius * t.sin();
            if !self.profile {
                self.plot_circle_pixels(x.round(), y.round(), x_c, y_c, &mut pixels);
            }
            t += t_step;
        }
        pixels
    }

    pub fn draw_bresenham(&self) -> Vec<Pos2> {
        let mut pixels = vec![];
        let (x_c, y_c) = (self.center.x, self.center.y);
        let mut x = 0i32;
        let mut y = self.radius as i32;
        let mut cap_delta: i32 = 2 * (1 - self.radius as i32);
        while y >= x {
            if !self.profile {
                self.plot_circle_pixels(x as f32, y as f32, x_c, y_c, &mut pixels);
            }
            if cap_delta < 0 {
                let delta = 2 * (cap_delta + y) - 1;
                if delta <= 0 {
                    x += 1;
                    cap_delta += 2 * x + 1;
                } else {
                    x += 1;
                    y -= 1;
                    cap_delta += 2 * (x - y + 1);
                }
            } else if cap_delta > 0 {
                let delta = 2 * (cap_delta - x) - 1;
                if delta <= 0 {
                    x += 1;
                    y -= 1;
                    cap_delta += 2 * (x - y + 1);
                } else {
                    y -= 1;
                    cap_delta -= 2 * y + 1;
                }
            } else {
                x += 1;
                y -= 1;
                cap_delta += 2 * (x - y + 1);
            }
        }

        pixels
    }

    pub fn draw_midpoint(&self) -> Vec<Pos2> {
        let mut pixels = vec![];
        let (x_c, y_c) = (self.center.x, self.center.y);
        let mut x = 0;
        let mut y = self.radius as i32;

        let mut trial: i32 = 4 * (5 - self.radius as i32);
        while x <= y {
            if !self.profile {
                self.plot_circle_pixels(x as f32, y as f32, x_c, y_c, &mut pixels);
            }
            x += 1;
            if trial > 0 {
                y -= 1;
                trial -= 8 * y;
            }
            trial += 8 * x + 4;
        }
        pixels
    }

    pub fn draw_builtin(&self) -> Vec<Pos2> {
        let _x = egui::Painter::circle_stroke(
            &mut egui::Painter::new(
                egui::Context::default(),
                egui::LayerId::debug(),
                egui::Rect::NAN,
            ),
            self.center.clone(),
            self.radius,
            egui::Stroke::new(1.0, egui::Color32::RED),
        );
        vec![]
    }
}

impl Measurable for Circle {
    fn measure_time(&mut self, draw_type: DrawType, rad: f32, _file: &std::fs::File) {
        let mut file = OpenOptions::new()
            .write(true)
            .open("/dev/null").unwrap();
        self.radius = rad;
        self.profile = true;
        let f = match draw_type {
            DrawType::CANONICAL => Circle::draw_canonic,
            DrawType::BRESENHAM => Circle::draw_bresenham,
            DrawType::PARAMETRIC => Circle::draw_parametric,
            DrawType::MIDPOINT => Circle::draw_midpoint,
            DrawType::BuiltIn => Circle::draw_builtin,
        };
        let start = std::time::Instant::now();
        let runs = 5000;
        for _ in 0..runs {
            f(self);
        }
        let n = start.elapsed().as_micros() / runs;
        file.write_all(format!("{} {}\n", rad, n).as_bytes()).unwrap();
    }
}

pub struct Ellipse {
    center: Pos2,
    radius: Vec2,
    profile: bool
}

impl Ellipse {
    pub fn new(center: Pos2, radius: Vec2) -> Self {
        Ellipse { center, radius, profile: false }
    }

    fn plot_ellipse_pixels(&self, x: f32, y: f32, x_c: f32, y_c: f32, pixels: &mut Vec<Pos2>) {
        let (mut sx, mut sy) = (1.0, 1.0);
        for _ in 0..4 {
            let xsx = x * sx;
            let ysy = y * sy;
            pixels.push(Pos2::new(x_c + xsx, y_c + ysy));
            sx *= -1.0;
            sy *= -1.0 * sx;
        }
    }

    pub fn draw_canonic(&self) -> Vec<Pos2> {
        let mut pixels = vec![];
        let (x_c, y_c) = (self.center.x, self.center.y);
        let sqr_a = self.radius.x * self.radius.x;
        let sqr_b = self.radius.y * self.radius.y;
        let sqrt_coeff = self.radius.y / self.radius.x;
        let hyp = (sqr_a + sqr_b).sqrt();
        let x_range = sqr_a / hyp + 1.0;
        let mut x = 0.0;
        while x <= x_range {
            let y = sqrt_coeff * (sqr_a - x * x).sqrt();
            if !self.profile {
                self.plot_ellipse_pixels(x, y.round(), x_c, y_c, &mut pixels);
            }
            x += 1.0;
        }

        let sqrt_coeff = sqrt_coeff.recip();
        let y_range = sqr_b / hyp + 1.0;
        let mut y = 0.0;
        while y <= y_range {
            let x = sqrt_coeff * (sqr_b - y * y).sqrt();
            self.plot_ellipse_pixels(x.round(), y, x_c, y_c, &mut pixels);
            y += 1.0;
        }

        pixels
    }

    pub fn draw_parametric(&self) -> Vec<Pos2> {
        let mut pixels = vec![];

        let (x_c, y_c) = (self.center.x, self.center.y);

        let t_range = std::f32::consts::FRAC_PI_2;
        let t_step = (if self.radius.x > self.radius.y {
            self.radius.x
        } else {
            self.radius.y
        })
        .recip();
        let mut t = 0.0;
        while t <= t_range {
            let x = self.radius.x * t.cos();
            let y = self.radius.y * t.sin();
            if !self.profile {
                self.plot_ellipse_pixels(x.round(), y.round(), x_c, y_c, &mut pixels);
            }
            t += t_step;
        }

        pixels
    }

    pub fn draw_bresenham(&self) -> Vec<Pos2> {
        let mut pixels = vec![];
        let (x_c, y_c) = (self.center.x, self.center.y);
        let (ra, rb) = (self.radius.x as i128, self.radius.y as i128);

        let sqr_a = ra * ra;
        let sqr_b = rb * rb;

        let mut x = 0;
        let mut y = rb;
        let mut cap_delta: i128 = sqr_b - sqr_a * (2 * rb - 1); // b^2(x+1)+a^2(y-1)^2-a^2b^2 в (0, b)

        while y >= 0 {
            if !self.profile {
                self.plot_ellipse_pixels(x as f32, y as f32, x_c, y_c, &mut pixels);
            }

            if cap_delta < 0 {
                let delta = 2 * cap_delta + sqr_a * (2 * y - 1);

                if delta <= 0 {
                    x += 1;
                    cap_delta += sqr_b * (2 * x + 1);
                } else {
                    x += 1;
                    y -= 1;
                    cap_delta += 2 * x * sqr_b - 2 * y * sqr_a + sqr_a + sqr_b;
                }
            } else if cap_delta > 0 {
                let delta = 2 * cap_delta - sqr_b * (2 * x + 1);

                if delta <= 0 {
                    x += 1;
                    y -= 1;
                    cap_delta += 2 * x * sqr_b - 2 * y * sqr_a + sqr_a + sqr_b;
                } else {
                    y -= 1;
                    cap_delta += sqr_a * (-2 * y + 1);
                }
            } else {
                x += 1;
                y -= 1;
                cap_delta += 2 * x * sqr_b - 2 * y * sqr_a + sqr_a + sqr_b;
            }
        }

        pixels
    }

    pub fn draw_midpoint(&self) -> Vec<Pos2> {
        let mut pixels = vec![];
        let (x_c, y_c) = (self.center.x, self.center.y);
        let (ra, rb) = (self.radius.x as i128, self.radius.y as i128);

        let sqr_a = ra * ra;
        let sqr_b = rb * rb;
        let sqr_a2 = 2 * sqr_a;
        let sqr_b2 = 2 * sqr_b;

        let mut x = 0;
        let mut y = rb;
        let mut trial: i128 = 2 * (sqr_b2 - sqr_a2 * (rb - 1));
        let mut dx = 0;
        let mut dy = sqr_a2 * y;

        // первый интервал
        while dx <= dy {
            if !self.profile {
                self.plot_ellipse_pixels(x as f32, y as f32, x_c, y_c, &mut pixels);
            }
            x += 1;
            dx += sqr_b2;

            if trial >= 0 {
                y -= 1;
                dy -= sqr_a2;
                trial -= 4 * dy;
            }

            trial += 4 * (dx + sqr_b);
        }

        // второй интервал
        trial -= 4 * (sqr_b * (x + 3) + sqr_a * (y - 3));
        while dy >= 0 {
            if !self.profile {
                self.plot_ellipse_pixels(x as f32, y as f32, x_c, y_c, &mut pixels);
            }
            y -= 1;
            dy -= sqr_a2;
            if trial < 0 {
                x += 1;
                dx += sqr_b2;
                trial += 4 * dx;
            }
            trial -= 4 * (dy - sqr_a);

        }
        pixels
    }

    pub fn draw_builtin(&self) -> Vec<Pos2> {
        let _x = egui::Painter::ellipse_stroke(
            &mut egui::Painter::new(
                egui::Context::default(),
                egui::LayerId::debug(),
                egui::Rect::NAN,
            ),
            self.center.clone(),
            self.radius,
            egui::Stroke::new(1.0, egui::Color32::RED),
        );
        vec![]
    }
}

impl Measurable for Ellipse {
    fn measure_time(&mut self, draw_type: DrawType, rad: f32, _file: &std::fs::File) {
        let mut file = OpenOptions::new()
            .write(true)
            .open("/dev/null").unwrap();
        self.radius = [rad, rad].into();
        self.profile = true;

        let f = match draw_type {
            DrawType::CANONICAL => Self::draw_canonic,
            DrawType::BRESENHAM => Self::draw_bresenham,
            DrawType::PARAMETRIC => Self::draw_parametric,
            DrawType::MIDPOINT => Self::draw_midpoint,
            DrawType::BuiltIn => Self::draw_builtin,
        };
        let runs = 2500;
        let start = std::time::Instant::now();
        for _ in 0..runs {
            f(self);
        }
        let n = start.elapsed().as_micros() / runs;
        file.write_all(format!("{} {}\n", rad, n).as_bytes()).unwrap();
    }
}
