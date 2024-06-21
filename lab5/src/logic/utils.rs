use eframe::egui::{Color32, Pos2};
use itertools::Itertools;
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy)]
pub struct EdgeInfo {
    x: f32,
    dy: f32,
    dx: f32,
}

impl Eq for EdgeInfo {}

impl PartialEq<Self> for EdgeInfo {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.dx == other.dx && self.dy == other.dy
    }
}

impl PartialOrd<Self> for EdgeInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EdgeInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.x
            .partial_cmp(&other.x)
            .unwrap_or(Ordering::Equal)
            .then_with(|| {
                self.dy
                    .partial_cmp(&other.dy)
                    .unwrap_or(Ordering::Equal)
                    .then_with(|| self.dx.partial_cmp(&other.dx).unwrap_or(Ordering::Equal))
            })
    }
}

#[derive(Debug)]
pub struct Canvas {
    filler: Vec<(Pos2, Pos2)>,
    edges: Vec<(usize, usize)>,

    last_closed: Vec<usize>,
    min_bound: Pos2,
    max_bound: Pos2,

    points: Vec<Pos2>,
    color32: Color32,
    active_edges: Vec<(f32, f32, f32)>,

    y_groups: HashMap<i32, BTreeSet<EdgeInfo>>,
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            filler: vec![],
            edges: vec![],
            last_closed: vec![0],
            min_bound: [f32::INFINITY; 2].into(),
            max_bound: [-f32::INFINITY; 2].into(),
            points: vec![],
            color32: Color32::WHITE,
            active_edges: vec![],
            y_groups: HashMap::new(),
        }
    }

    pub fn last_closed(&self) -> usize {
        self.last_closed.last().unwrap().to_owned()
    }

    pub fn points_rem(&mut self, c: usize) {
        self.points.remove(c);
    }

    pub fn all_closed(&self) -> &[usize] {
        &self.last_closed
    }

    pub fn last_closed_point(&self) -> Option<&Pos2> {
        let last = self.last_closed.last();
        let last = last?;
        self.points.get(last.to_owned())
    }

    pub fn set_color(&mut self, color32: Color32) {
        self.color32 = color32;
    }

    pub fn fill_string(&mut self, pos1: Pos2, pos2: Pos2) {
        self.filler.push((pos1, pos2));
    }

    pub fn filler(&self) -> &[(Pos2, Pos2)] {
        &self.filler
    }

    pub fn edges(&self) -> &[(usize, usize)] {
        &self.edges
    }

    pub fn points(&self) -> &[Pos2] {
        &self.points
    }

    pub fn add_line(&mut self, a: usize, b: usize) {
        self.edges.push((a, b));
    }

    pub fn close(&mut self) -> Option<()> {
        if self.points.len() - self.last_closed() >= 3 {
            self.add_line(self.last_closed(), self.points.len() - 1);
            self.last_closed.push(self.points.len());
            Some(())
        } else {
            None
        }
    }

    pub fn is_closed(&self) -> bool {
        self.points.len() - self.last_closed() >= 2
    }

    pub fn clear(&mut self) {
        self.points.clear();
        self.last_closed.clear();
        self.last_closed.push(0);
        self.min_bound = [f32::INFINITY; 2].into();
        self.max_bound = [-f32::INFINITY; 2].into();
        self.edges.clear();
        self.filler.clear();
        self.y_groups.clear();
        self.active_edges.clear();
    }

    pub fn clean(&mut self) {
        self.filler.clear();
        self.active_edges.clear();
        self.y_groups.clear();
        self.update_y_group(0);
    }

    // pub fn test_figure(&self) -> bool {
    //     self.min_bound.y == self.max_bound.y || self.min_bound.x == self.max_bound.x
    // }

    pub fn add_point(&mut self, pos2: Pos2) {
        let last = self.last_closed();
        if self.points.len() > last && self.points[last] == pos2 {
            self.close();
        } else {
            self.update_bounds(pos2.clone());
            self.points.push(pos2.clone());
            let len = self.points.len();
            if len - last >= 2 {
                self.add_line(len - 2, len - 1);
            }
        }
    }

    fn update_bounds(&mut self, pos2: Pos2) {
        self.min_bound.x = self.min_bound.x.min(pos2.x);
        self.min_bound.y = self.min_bound.y.min(pos2.y);

        self.max_bound.x = self.max_bound.x.max(pos2.x);
        self.max_bound.y = self.max_bound.y.max(pos2.y);
    }

    fn create_y_groups(&mut self) {
        fn get_edge(mut begin: Pos2, mut end: Pos2) -> Option<(i32, EdgeInfo)> {
            if begin.y > end.y {
                std::mem::swap(&mut end, &mut begin)
            }
            let dy = end.y - begin.y;
            if dy == 0.0 {
                None
            } else {
                Some((
                    end.y as i32,
                    EdgeInfo {
                        x: end.x,
                        dy,
                        dx: (begin.x - end.x) / dy,
                    },
                ))
            }
        }
        self.last_closed.windows(2).for_each(|pair| {
            let edges = self.points[pair[0]..pair[1]]
                .windows(2)
                .flat_map(|window| window.iter().zip(window.iter().skip(1)))
                .filter_map(|(p1, p2)| get_edge(*p1, *p2))
                .chain(get_edge(self.points[pair[0]], self.points[pair[1] - 1]));
            edges.for_each(|(y, edge)| {
                self.y_groups
                    .entry(y)
                    .or_insert_with(BTreeSet::new)
                    .insert(edge);
            });
        });
    }

    fn update_y_group(&mut self, y: i32) {
        if let Some((_, edges)) = self.y_groups.remove_entry(&(y + 1)) {
            let edges = edges.into_iter().filter_map(|edge| {
                if edge.dy != 1.0 {
                    Some(EdgeInfo {
                        x: edge.x + edge.dx,
                        dy: edge.dy - 1.0,
                        dx: edge.dx,
                    })
                } else {
                    None
                }
            });
            self.y_groups
                .entry(y)
                .or_insert_with(BTreeSet::new)
                .extend(edges);
        }
    }

    pub fn filling(canvas: &Arc<Mutex<Self>>, dur: &mut Arc<Mutex<std::time::Duration>>, delay: u64) {
        let start = std::time::Instant::now();
        let mut canvas_locked = canvas.lock().unwrap();
        canvas_locked.create_y_groups();
        let (min_y, max_y) = (
            canvas_locked.min_bound.y as i32,
            canvas_locked.max_bound.y as i32,
        );
        drop(canvas_locked);
        for y in (min_y + 1..=max_y).rev() {
            let cur_y_group = canvas
                .lock()
                .unwrap()
                .y_groups
                .get(&y)
                .unwrap_or(&BTreeSet::new())
                .clone()
                .into_iter();
            for mut line in &cur_y_group.chunks(2) {
                let line1 = line.next().unwrap();
                let line2 = line.next().unwrap();
                let p1 = Pos2::new(line1.x, y as f32);
                let p2 = Pos2::new(line2.x, y as f32);
                std::thread::sleep(std::time::Duration::from_millis(delay));
                canvas.lock().unwrap().fill_string(p1.round(), p2.round());
            }
            *dur.lock().unwrap() = start.elapsed();
            canvas.lock().unwrap().update_y_group(y - 1);
        }
    }
}
