use std::collections::HashMap;
use nalgebra::Scale;
use std::f32::consts::PI;

pub struct Spacing {
    b: f32,
    e: f32,
    d: f32,
    amount: usize,
}

impl Spacing {
    pub fn new(min: f32, max: f32, step: f32) -> Self {
        let amount = ((max - min) / step).ceil() as usize;
        Self { b: min, e: max, d: step, amount }
    }
}

pub struct Horizont {
    screen_size: (usize, usize),
    down: HashMap<isize, isize>,
    top: HashMap<isize, isize>,
    f: fn(f32, f32) -> f32,
    scale: f32,
}

impl Horizont {
    pub fn new(screen_size: (usize, usize), f: fn(f32, f32) -> f32, scale: f32) -> Self {
        let (width, _) = screen_size;
        let down = HashMap::new();
        let top = HashMap::new();
        Self { screen_size, down, top, f, scale }
    }

    fn prepare_arrays(&mut self) {
        let (width, height) = self.screen_size;
        self.down = HashMap::new();
        self.top = HashMap::new();
        for x in 0..width {
            self.down.insert(x as isize, height as isize);
            self.top.insert(x as isize, 0);
        }
    }

    fn get_intersection(&self, x1: isize, y1: isize, x2: isize, y2: isize, horizon: &HashMap<isize, isize>, xi: &mut isize, yi: &mut isize) {
        *xi = x1;
        *yi = y1;
        let delta_x = x2 - x1;
        let delta_y_c = y2 - y1;
        let delta_y_p = horizon.get(&x2).unwrap_or(&0) - horizon.get(&x1).unwrap_or(&0);
        let m = delta_y_c as f32 / delta_x as f32;
        if delta_x == 0 {
            *xi = x2;
            *yi = *horizon.get(&x2).unwrap_or(&0);
        } else if y1 == *horizon.get(&x1).unwrap_or(&0) && y2 == *horizon.get(&x2).unwrap_or(&0) {
            *xi = x1;
            *yi = y1;
        } else {
            *xi = x1 - ((delta_x as f32 * (y1 - *horizon.get(&x1).unwrap_or(&0)) as f32 / (delta_y_c as f32 - delta_y_p as f32)).round() as isize);
            *yi = ((*xi - x1) as f32 * m + y1 as f32).round() as isize;
        }
        // if (*yi as f32 - m - *horizon.get(&(*xi - 1)).unwrap() as f32).abs() <= (*yi as f32 - m - *horizon.get(xi).unwrap_or(&0) as f32).abs() {
        //     *yi = (*yi as f32 - m) as isize;
        //     *xi = *xi - 1;
        // }
        // if (x2 - x1) == 0 {
        //     *xi = x2;
        //     *yi = *horizon.get(&x2).unwrap();
        // } else {
        //     let m = (y2 - y1) as f32 / (x2 - x1) as f32;
            // let y_sign = y1 as f32 + m - *horizon.get(&(x1 + 1)).unwrap() as f32;
            // let y_sign = y_sign.signum();
            // let mut c_sign = y_sign.signum();
            // *yi = (y1 as f32 + m) as isize;
            // *xi = x1 + 1;
            // while c_sign == y_sign {
            //     *yi = (*yi as f32 + m) as isize;
            //     *xi = *xi + 1;
            //     c_sign = (*yi as f32 - *horizon.get(xi).unwrap_or(&0) as f32).signum();
            // }
    }

    fn horizon(&mut self, mut x1: isize, mut y1: isize, mut x2: isize, mut y2: isize, painter: &mut Vec<(isize, isize, isize, isize)>) {
        let (width, _) = self.screen_size;
        if x2 < 0 || x2 >= width as isize || x1 < 0 || x1 >= width as isize {
            return;
        }
        if x2 < x1 {
            Self::swap(&mut x1, &mut x2);
            Self::swap(&mut y1, &mut y2);
        }
        if x2 - x1 == 0 {
            *self.top.entry(x2).or_insert(0) = self.top[&x2].max(y2);
            *self.down.entry(x2).or_insert(width as isize) = self.down[&x2].min(y2);
            if x2 >= 0 && x2 <= width as isize {
                painter.push((x1, y1, x2, y2));
            }
        } else {
            let mut x_prev = x1;
            let mut y_prev = y1;
            let m = (y2 - y1) as f32 / (x2 - x1) as f32;
            for x in x1..=x2 {
                let y = (m * (x - x1) as f32 + y1 as f32).round() as isize;
                *self.top.entry(x).or_insert(0) = self.top[&x].max(y);
                *self.down.entry(x).or_insert(width as isize) = self.down[&x].min(y);
                if x >= 0 && x < width as isize {
                    painter.push((x_prev, y_prev, x, y));
                }
                x_prev = x;
                y_prev = y;
            }
        }
    }

    fn process_edge(&mut self, x: &mut isize, y: &mut isize, x_edge: &mut isize, y_edge: &mut isize, painter: &mut Vec<(isize, isize, isize, isize)>) {
        if *x_edge != -1 {
            self.horizon(*x_edge, *y_edge, *x, *y, painter);
        }
        *x_edge = *x;
        *y_edge = *y;
    }

    fn visible(&self, x: isize, y: isize) -> isize {
        if y < *self.top.get(&x).unwrap_or(&0) && y > *self.down.get(&x).unwrap_or(&(self.screen_size.1 as isize)) {
            0
        } else if y >= *self.top.get(&x).unwrap_or(&0) {
            1
        } else {
            -1
        }
    }

    fn rotate_x(y: &mut f32, z: &mut f32, tetax: f32) {
        let tetax = tetax * PI / 180.0;
        let buf = *y;
        *y = tetax.cos() * *y + tetax.sin() * *z;
        *z = tetax.cos() * *z + tetax.sin() * buf;
    }

    fn rotate_y(x: &mut f32, z: &mut f32, tetay: f32) {
        let tetay = tetay * PI / 180.0;
        let buf = *x;
        *x = tetay.cos() * *x - tetay.sin() * *z;
        *z = tetay.sin() * buf + tetay.cos() * *z;
    }

    fn rotate_z(x: &mut f32, y: &mut f32, tetaz: f32) {
        let tetaz = tetaz * PI / 180.0;
        let buf = *x;
        *x = tetaz.cos() * *x - tetaz.sin() * *y;
        *y = tetaz.sin() * buf + tetaz.cos() * *y;
    }

    fn transform(&self, x: &mut f32, y: &mut f32, z: &mut f32, tetax: f32, tetay: f32, tetaz: f32, res_x: &mut isize, res_y: &mut isize, coef: f32) {
        let (width, height) = self.screen_size;
        let x_center = width as f32 / 2.0;
        let y_center = height as f32 / 2.0;
        let mut x_tmp = *x;
        let mut y_tmp = *y;
        let mut z_tmp = *z;
        Self::rotate_x(&mut y_tmp, &mut z_tmp, tetax);
        Self::rotate_y(&mut x_tmp, &mut z_tmp, tetay);
        Self::rotate_z(&mut x_tmp, &mut y_tmp, tetaz);
        *res_x = (x_tmp * self.scale + x_center).round() as isize;
        *res_y = (y_tmp * self.scale + y_center).round() as isize;
    }

    pub fn horizon_algo(&mut self, mut par_x: Spacing, par_z: Spacing, painter: &mut Vec<(isize, isize, isize, isize)>, tetax: f32, tetay: f32, tetaz: f32, coef: f32) {
        self.prepare_arrays();

        let mut x_left = -1;
        let mut y_left = -1;
        let mut x_right = -1;
        let mut y_right = -1;
        let mut x_prev = 0;
        let mut y_prev = 0;
        let mut z = par_z.e;
        while z >= par_z.b {
            let mut y_p = (self.f)(par_x.b, z as f32);
            self.transform(&mut par_x.b, &mut y_p, &mut (z as f32), tetax, tetay, tetaz, &mut x_prev, &mut y_prev, self.scale);
            self.process_edge(&mut x_prev, &mut y_prev, &mut x_left, &mut y_left, painter);
            let mut p_flag = self.visible(x_prev, y_prev);
            let mut x = par_x.b;
            while x <= par_x.e {
                let mut x_curr = 0;
                let mut y_curr = 0;
                let mut xi = 0;
                let mut yi = 0;
                y_p = (self.f)(x as f32, z as f32);
                self.transform(&mut (x as f32), &mut y_p, &mut (z as f32), tetax, tetay, tetaz, &mut x_curr, &mut y_curr, self.scale);
                if self.top.get(&x_curr).is_none() || self.down.get(&x_curr).is_none() {
                    x += par_x.d;
                    continue;
                }
                let t_flag = self.visible(x_curr, y_curr);
                if t_flag == p_flag {
                    if p_flag != 0 {
                        self.horizon(x_prev, y_prev, x_curr, y_curr, painter);
                    }
                } else if t_flag == 0 {
                    if p_flag == 1 {
                        self.get_intersection(x_prev, y_prev, x_curr, y_curr, &self.top, &mut xi, &mut yi);
                    } else {
                        self.get_intersection(x_prev, y_prev, x_curr, y_curr, &self.down, &mut xi, &mut yi);
                    }
                    self.horizon(x_prev, y_prev, xi, yi, painter);
                } else if t_flag == 1 {
                    if p_flag == 0 {
                        self.get_intersection(x_prev, y_prev, x_curr, y_curr, &self.top, &mut xi, &mut yi);
                        self.horizon(x_prev, y_prev, xi, yi, painter);
                    } else {
                        self.get_intersection(x_prev, y_prev, x_curr, y_curr, &self.top, &mut xi, &mut yi);
                        self.horizon(x_prev, y_prev, xi, yi, painter);
                        self.get_intersection(x_prev, y_prev, x_curr, y_curr, &self.down, &mut xi, &mut yi);
                        self.horizon(xi, yi, x_curr, y_curr, painter);
                    }
                } else {
                    if p_flag == 0 {
                        self.get_intersection(x_prev, y_prev, x_curr, y_curr, &self.down, &mut xi, &mut yi);
                        self.horizon(x_prev, y_prev, xi, yi, painter);
                    } else {
                        self.get_intersection(x_prev, y_prev, x_curr, y_curr, &self.top, &mut xi, &mut yi);
                        self.horizon(x_prev, y_prev, xi, yi, painter);
                        self.get_intersection(x_prev, y_prev, x_curr, y_curr, &self.down, &mut xi, &mut yi);
                        self.horizon(xi, yi, x_curr, y_curr, painter);
                    }
                }
                p_flag = t_flag;
                x_prev = x_curr;
                y_prev = y_curr;

                x += par_x.d;
            }
            self.process_edge(&mut x_prev, &mut y_prev, &mut x_right, &mut y_right, painter);
            z -= par_z.d;
        }
    }

    fn swap<T>(a: &mut T, b: &mut T) {
        std::mem::swap(a, b);
    }
}
