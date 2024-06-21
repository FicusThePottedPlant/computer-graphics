use eframe::egui::{Color32, Pos2};

fn get_code(rect: (Pos2, Pos2), a: Pos2) -> u8 {
    let (left, right) = rect;
    let mut code: u8 = 0;
    if a.x < left.x {
        code += 8;
    }
    if a.x > right.x {
        code += 4;
    }
    if a.y > right.y {
        code += 2;
    }
    if a.y < left.y {
        code += 1;
    }
    code
}

fn find_r_point(start: Pos2, end: Pos2, cutter_l: Pos2, cutter_r: Pos2) -> Option<Pos2> {
    let q = start;
    let m = if start.x != end.x { (end.y - start.y) / (end.x - start.x) } else { f32::INFINITY };


    if cutter_l.x >= q.x && m.is_finite() {
        let y = m * (cutter_l.x - q.x) + q.y;
        if y >= cutter_l.y && y <= cutter_r.y  {
            return Some(Pos2 {x: cutter_l.x, y});
        }
    }

    if cutter_r.x <= q.x && m.is_finite() {
        let y = m * (cutter_r.x - q.x) + q.y;
        if y >= cutter_l.y && y <= cutter_r.y  {
            return Some(Pos2 {x: cutter_r.x, y});
        }
    }

    if m == 0.0 {
        return None;
    }

    if cutter_l.y >= q.y {
        let x = (cutter_l.y - q.y) / m + q.x;
        if x >= cutter_l.x && x <= cutter_r.x  {
            return Some(Pos2 { x, y: cutter_l.y });
        }
    }

    if cutter_r.y <= q.y {
        let x = (cutter_r.y - q.y) / m + q.x;
        if x >= cutter_l.x && x <= cutter_r.x  {
            return Some(Pos2 { x, y: cutter_r.y });
        }
    }

    None
}


pub fn cut(rect: (Pos2, Pos2), lines: &Vec<(Pos2, Pos2)>) -> Vec<(Pos2, Pos2)> {
    let (cutter_l, cutter_r) = rect;
    lines.iter().filter_map(|&(p1, p2)| {
        let (code1, code2) = (get_code(rect, p1), get_code(rect, p2));
        match (code1, code2) {
            (0, 0) => Some((p1, p2)),
            (c1, c2) if c1 & c2 == 0 => {
                let (mut r1, mut r2) = (p1, p2);
                if c1 != 0 {
                    r1 = find_r_point(p1, p2, cutter_l, cutter_r)?;
                }
                if c2 != 0 {
                    r2 = find_r_point(p2, p1, cutter_l, cutter_r)?;
                }
                Some((r1, r2))
            },
            _ => None,
        }
    }).collect()
}
