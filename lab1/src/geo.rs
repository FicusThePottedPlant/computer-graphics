use itertools::{iproduct, Itertools};
use std::f64::consts::PI;

#[derive(Debug, Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
#[derive(Debug)]
pub enum LineType {
    Tan(f64, f64),
    OnlyX(f64),
}
use LineType::*;

#[derive(Debug)]
struct Line {
    a: Point,
    b: Point,
}
impl Line {
    pub fn new(a: &Point, b: &Point) -> Self {
        Self {
            a: a.clone(),
            b: b.clone(),
        }
    }
}

impl Point {
    pub fn new(x: impl Into<f64>, y: impl Into<f64>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }

    pub fn distance(first: &Point, second: &Point) -> f64 {
        let dx = second.x - first.x;
        let dy = second.y - first.y;
        let distance = (dx * dx + dy * dy).sqrt();
        distance
    }

    fn calculate_bisect(first: &Point, second: &Point, third: &Point) -> Line {
        let (ab, ad) = (Self::distance(first, second), Self::distance(first, third));
        Line {
            a: first.to_owned(),
            b: Self {
                x: second.x + ab / (ab + ad) * (third.x - second.x),
                y: second.y + ab / (ab + ad) * (third.y - second.y),
            },
        }
    }

    fn line(first: &Point, second: &Point) -> LineType {
        if (second.x - first.x).abs() <= 1e-6 {
            return OnlyX(second.x);
        }
        let k = (second.y - first.y) / (second.x - first.x);
        let b = first.y - k * first.x;
        Tan(k, b)
    }

    fn calculate_intersection(
        first: &Point,
        second: &Point,
        third: &Point,
        fourth: &Point,
    ) -> Option<Self> {
        let line1 = Self::line(first, second);
        let line2 = Self::line(third, fourth);
        match line1 {
            Tan(k1, b1) => match line2 {
                Tan(k2, b2) => {
                    if b2 != b1 {
                        let x = (b2 - b1) / (k1 - k2);
                        return Some(Self { x, y: k1 * x + b1 });
                    }
                    None
                }
                OnlyX(x) => Some(Self { x, y: k1 * x + b1 }),
            },
            OnlyX(x) => match line2 {
                OnlyX(_) => None,
                Tan(k, b) => Some(Self { x, y: k * x + b }),
            },
        }
    }
}
pub fn point_form_triangle(first: &Point, second: &Point, third: &Point) -> bool {
    let (ab, ad, cd) = (
        Point::distance(first, second),
        Point::distance(first, third),
        Point::distance(second, third),
    );
    form_triangle(ab, ad, cd)
}

pub fn form_triangle(first: f64, second: f64, third: f64) -> bool {
    first + second >= third && first + third >= second && second + third >= first
}

pub fn ord_angle(line: LineType) -> f64 {
    match line {
        Tan(k, _) => k.atan(),
        OnlyX(_) => 0f64,
    }
}

fn line_fmt(line: &Line) -> LineType {
    if (line.b.x - line.a.x).abs() <= 1e-6 {
        return OnlyX(line.a.x);
    }
    let k = (line.b.y - line.a.y) / (line.b.x - line.a.x);
    let b = line.a.y - k * line.a.x;
    Tan(k, b)
}
#[derive(Debug)]
pub struct Res {
    tria1: (Line, Line, Line),
    bis1: (Line, Line, Line),
    tria2: (Line, Line, Line),
    bis2: (Line, Line, Line),
    con: Line,
}

pub fn calc(a: &Vec<Point>, b: &Vec<Point>) -> Option<Res> {
    let mut max_angle = (2.0 * PI, None);
    for (x, y) in iproduct!(a.iter().combinations(3), b.iter().combinations(3)) {
        let (x0, x1, x2) = (x[0], x[1], x[2]);
        let (y0, y1, y2) = (y[0], y[1], y[2]);

        if point_form_triangle(x0, x1, x2) && point_form_triangle(y0, y1, y2) {
            let bis_a = Point::calculate_bisect(x0, x1, x2);
            let bis_b = Point::calculate_bisect(x1, x2, x0);
            let bis_c = Point::calculate_bisect(x2, x0, x1);
            let c1 = Point::calculate_intersection(&bis_a.a, &bis_a.b, &bis_b.a, &bis_b.b);

            let bis_a2 = Point::calculate_bisect(y0, y1, y2);
            let bis_b2 = Point::calculate_bisect(y1, y2, y0);
            let bis_c2 = Point::calculate_bisect(y2, y0, y1);
            let c2 = Point::calculate_intersection(&bis_a2.a, &bis_a2.b, &bis_b2.a, &bis_b2.b);

            println!("{:?} {:?}", c1, c2);
            let new_line = if let (Some(first), Some(second)) = (c1, c2) {
                Some(Line {
                    a: first,
                    b: second,
                })
            } else {
                None
            };

            if let Some(line) = new_line {
                let new_angle = ord_angle(line_fmt(&line));
                if new_angle <= max_angle.0 {
                    max_angle = (new_angle, Some(Res {
                        tria1: (Line::new(x0, x1), Line::new(x1, x2), Line::new(x2, x0)),
                        tria2: (Line::new(y0, y1), Line::new(y1, y2), Line::new(y2, y0)),
                        bis1: (bis_a, bis_b, bis_c),
                        bis2: (bis_a2, bis_b2, bis_c2),
                        con: line,
                    }));
                }
            }
        }
    }
    max_angle.1
}
