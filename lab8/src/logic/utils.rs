use eframe::egui::{Pos2, pos2, vec2, Vec2};

#[derive(Debug, Default)]
pub struct Polygon {
    vertices: Vec<Pos2>,
    closed: bool,
}

impl Polygon {
    pub fn push(&mut self, pos2: Pos2) -> &mut Self {
        if let Some(&x) = self.vertices.last() {
            if x == pos2 {
                return self;
            }
        }
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

pub fn cut_one(rect: &[Pos2], line: &(Pos2, Pos2)) -> Option<(Pos2, Pos2)> {
    let d = line.1 - line.0;
    let count = rect.len();

    (0..count)
        .try_fold((0.0f32, 1.0f32), |(top, bottom), i| {
            let norm = normal(rect[i], rect[(i + 1) % count], rect[(i + 2) % count]);
            let w = line.0 - rect[i];

            let d_scal = scalar(d, norm);
            let w_scal = scalar(w, norm);

            match d_scal {
                0.0 => {
                    if w_scal < 0.0 {
                        Err(())
                    } else {
                        Ok((top, bottom))
                    }
                }
                _ if d_scal > 0.0 => {
                    let param = -w_scal / d_scal;
                    if param <= 1.0 {
                        Ok((top.max(param), bottom))
                    } else {
                        Err(())
                    }
                }
                _ => {
                    let param = -w_scal / d_scal;
                    if param >= 0.0 {
                        Ok((top, bottom.min(param)))
                    } else {
                        Err(())
                    }
                }
            }
        })
        .ok()
        .and_then(|(top, bottom)| {
            if top <= bottom {
                Some((
                    pos2(line.0.x + d.x * top, line.0.y + d.y * top),
                    pos2(line.0.x + d.x * bottom, line.0.y + d.y * bottom),
                ))
            } else {
                None
            }
        })
}


fn check_convexity_polygon(cutter: &[Pos2]) -> bool {
    if cutter.len() < 3 {
        return false;
    }

    let vect1 = cutter[0] - cutter[1];
    let vect2 = cutter[1] - cutter[2];

    let sign = if cross(vect1, vect2) > 0.0 { 1.0 } else { -1.0 };

    (0..cutter.len()).all(|i| {
        let vect_i = cutter[(i + cutter.len() - 2) % cutter.len()] - cutter[(i + cutter.len() - 1) % cutter.len()];
        let vect_j = cutter[(i + cutter.len() - 1) % cutter.len()] - cutter[i];
        sign * cross(vect_i, vect_j) >= 0.0
    })
}

pub fn cut(rect: &[Pos2], lines: &[(Pos2, Pos2)]) -> Option<Vec<(Pos2, Pos2)>> {
    if rect.len() < 3 {
        return None;
    }
    let rect = &rect[0..rect.len() - 1]; // slice last repeated
    if check_convexity_polygon(rect) {
        Some(lines.iter()
            .filter_map(|line| cut_one(rect, line))
            .collect())
    } else {
        None
    }
}