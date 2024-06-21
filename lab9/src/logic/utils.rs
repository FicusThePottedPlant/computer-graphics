use eframe::egui::{Pos2, pos2, vec2, Vec2};
use nalgebra::{Matrix2, Vector2};

#[derive(Debug, Default)]
pub struct Polygon {
    vertices: Vec<Pos2>,
    closed: bool,
}

impl Polygon {
    pub fn push(&mut self, pos2: Pos2) -> &mut Self {
        self.vertices.push(pos2);
        self
    }

    pub fn vertices(&mut self) -> &[Pos2] {
        &self.vertices
    }

    pub fn last(&mut self) -> Option<&Pos2> {
        self.vertices.last()
    }

    pub fn clear(&mut self) -> &mut Self {
        self.vertices.clear();
        self.open();
        self.closed = false;
        self
    }

    pub fn closed(&self) -> bool {
        self.closed
    }

    pub fn close(&mut self) -> &mut Self {
        if self.vertices.len() < 3 || self.closed == true {
            return self;
        }
        self.vertices.push(self.vertices[0]);
        self.closed = true;
        self
    }

    pub fn open(&mut self) -> &mut Self {
        self.vertices.clear();
        self.closed = false;
        self
    }
}

fn scalar(a: Vec2, b: Vec2) -> f32 {
    a.x * b.x + a.y * b.y
}

fn cross(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

fn normal(a: Pos2, b: Pos2, pos: Pos2) -> Vec2 {
    let fvec = b - a;
    let posvec = pos - b;
    let mut normvec = if fvec.y != 0.0 {
        vec2(1.0, -fvec.x / fvec.y)
    } else {
        vec2(0.0, 1.0)
    };

    if scalar(posvec, normvec) < 0.0 {
        normvec *= -1.0;
    }
    normvec
}

fn visibility(point: Pos2, begin: Pos2, end: Pos2) -> f32 {
    let res = (point.x - begin.x) * (end.y - begin.y) - (point.y - begin.y) * (end.x - begin.x);

    if res.abs() < f32::EPSILON {
        0.0
    } else {
        res.signum()
    }
}

fn check_lines_crossing(begin1: Pos2, end1: Pos2, begin2: Pos2, end2: Pos2) -> bool {
    let vis1 = visibility(begin1, begin2, end2);
    let vis2 = visibility(end1, begin2, end2);

    (vis1 < 0.0 && vis2 > 0.0) || (vis1 > 0.0 && vis2 < 0.0)
}

fn get_cross_point(begin1: Pos2, end1: Pos2, begin2: Pos2, end2: Pos2) -> Pos2 {
    let coef = Matrix2::new(
        end1.x - begin1.x, begin2.x - end2.x,
        end1.y - begin1.y, begin2.y - end2.y,
    );

    let rights = Vector2::new(
        begin2.x - begin1.x,
        begin2.y - begin1.y,
    );

    let param = coef.try_inverse().unwrap() * rights;

    let x = begin1.x + (end1.x - begin1.x) * param.x;
    let y = begin1.y + (end1.y - begin1.y) * param.x;

    Pos2::new(x, y)
}


fn is_inside(p: &Pos2, cp1: &Pos2, cp2: &Pos2) -> bool {
    (cp2.x - cp1.x) * (p.y - cp1.y) > (cp2.y - cp1.y) * (p.x - cp1.x)
}

fn compute_intersection(cp1: &Pos2, cp2: &Pos2, s: &Pos2, e: &Pos2) -> Pos2 {
    let dc = Pos2 {
        x: cp1.x - cp2.x,
        y: cp1.y - cp2.y,
    };
    let dp = Pos2 {
        x: s.x - e.x,
        y: s.y - e.y,
    };
    let n1 = cp1.x * cp2.y - cp1.y * cp2.x;
    let n2 = s.x * e.y - s.y * e.x;
    let n3 = 1.0 / (dc.x * dp.y - dc.y * dp.x);
    Pos2 {
        x: (n1 * dp.x - n2 * dc.x) * n3,
        y: (n1 * dp.y - n2 * dc.y) * n3,
    }
}


fn check_convexity_polygon(cutter: &mut Vec<Pos2>) -> bool {
    if cutter.len() < 3 {
        return false;
    }

    let vect1 = cutter[0] - cutter[1];
    let vect2 = cutter[1] - cutter[2];

    let mut sign = if cross(vect1, vect2) > 0.0 { 1.0 } else { -1.0 };
    if sign < 0.0 {
        cutter.reverse();
        sign *= -1.;
    }
    (0..cutter.len()).all(|i| {
        let vect_i = cutter[(i + cutter.len() - 2) % cutter.len()] - cutter[(i + cutter.len() - 1) % cutter.len()];
        let vect_j = cutter[(i + cutter.len() - 1) % cutter.len()] - cutter[i];
        sign * cross(vect_i, vect_j) >= 0.0
    })
}

pub fn cut_one(clipper: &[Pos2], polygon: &[Pos2]) -> Vec<Pos2> {
    if polygon.is_empty() {
        return Vec::new();
    }
    let res = clipper
        .windows(2)
        .try_fold(polygon.to_vec(), |p, window| {
            let (w_start, w_end) = (window[0], window[1]);
            let mut q = Vec::new();
            let mut s = p.last().unwrap().clone();

            for &point in &p {
                if check_lines_crossing(s, point, w_start, w_end) {
                    q.push(get_cross_point(s, point, w_start, w_end));
                }
                if visibility(point, w_start, w_end) <= 0.0 {
                    q.push(point);
                }
                s = point;
            }

            if check_lines_crossing(s, p[0], w_start, w_end) {
                q.push(get_cross_point(s, p[0], w_start, w_end));
            }
            if q.is_empty() {
                None
            } else {
                Some(q)
            }
        });
    res.unwrap_or(Vec::new())
}


pub fn cut(cutter: &[Pos2], polygon: &[Pos2]) -> Option<Vec<Pos2>> {
    if cutter.len() < 3 {
        return None;
    }
    let mut polygon = polygon.to_vec();
    let mut cutter = cutter.to_vec();
    if check_convexity_polygon(&mut cutter) {
        cutter.push(cutter[0]);
        let mut res = cut_one(&cutter, &polygon);
        if res.len() > 0 {
            res.push(res[0]);
        }
        Some(res)
    } else {
        None
    }
}
