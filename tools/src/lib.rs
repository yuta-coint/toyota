#![allow(non_snake_case, unused_macros)]

use itertools::Itertools;
use proconio::input;
use rand::prelude::*;
use svg::node::{
    element::{Group, Line, Rectangle, Style, Title},
    Text,
};

pub trait SetMinMax {
    fn setmin(&mut self, v: Self) -> bool;
    fn setmax(&mut self, v: Self) -> bool;
}
impl<T> SetMinMax for T
where
    T: PartialOrd,
{
    fn setmin(&mut self, v: T) -> bool {
        *self > v && {
            *self = v;
            true
        }
    }
    fn setmax(&mut self, v: T) -> bool {
        *self < v && {
            *self = v;
            true
        }
    }
}

#[macro_export]
macro_rules! mat {
	($($e:expr),*) => { Vec::from(vec![$($e),*]) };
	($($e:expr,)*) => { Vec::from(vec![$($e),*]) };
	($e:expr; $d:expr) => { Vec::from(vec![$e; $d]) };
	($e:expr; $d:expr $(; $ds:expr)+) => { Vec::from(vec![mat![$e $(; $ds)*]; $d]) };
}

#[derive(Clone, Debug)]
pub struct Input {
    pub D: usize,
    pub rs: Vec<(usize, usize)>,
    pub ts: Vec<usize>,
}

impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {}", self.D, self.rs.len())?;
        for &(i, j) in &self.rs {
            writeln!(f, "{} {}", i, j)?;
        }
        for &i in &self.ts {
            writeln!(f, "{}", i)?;
        }
        Ok(())
    }
}

pub fn parse_input(f: &str) -> Input {
    let f = proconio::source::once::OnceSource::from(f);
    input! {
        from f,
        D: usize, N: usize,
        rs: [(usize, usize); N],
        ts: [usize; D * D - 1 - N],
    }
    Input { D, rs, ts }
}

pub fn read<T: Copy + PartialOrd + std::fmt::Display + std::str::FromStr>(
    token: Option<&str>,
    lb: T,
    ub: T,
) -> Result<T, String> {
    if let Some(v) = token {
        if let Ok(v) = v.parse::<T>() {
            if v < lb || ub < v {
                Err(format!("Out of range: {}", v))
            } else {
                Ok(v)
            }
        } else {
            Err(format!("Parse error: {}", v))
        }
    } else {
        Err("Unexpected EOF".to_owned())
    }
}

pub struct Output {
    pub actions: Vec<(usize, usize)>,
    pub comments: Vec<String>,
}

pub fn parse_output(input: &Input, f: &str) -> Result<Output, String> {
    let mut actions = vec![];
    let mut comments = vec![];
    let mut comment = String::new();
    for v in f.lines() {
        let v = v.trim();
        if v.len() == 0 {
            continue;
        } else if v.starts_with("#") {
            comment += v.trim_start_matches("#");
            comment.push('\n');
        } else {
            let mut tokens = v.split_whitespace();
            actions.push((read(tokens.next(), 0, input.D)?, read(tokens.next(), 0, input.D)?));
            if tokens.next().is_some() {
                return Err(format!("Illegal output: {}", v));
            }
            comments.push(comment);
            comment = String::new();
        }
    }
    if actions.len() > 2 * (input.D * input.D - 1) {
        return Err(format!("Too many output"));
    }
    Ok(Output { actions, comments })
}

pub fn gen(seed: u64, fix_D: Option<usize>, fix_N: Option<usize>) -> Input {
    let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed ^ 6);
    let mut D = 9;
    if let Some(fix_D) = fix_D {
        D = fix_D;
    }
    let mut N = rng.gen_range(0..=D as i32) as usize;
    if let Some(fix_N) = fix_N {
        N = fix_N;
    }
    let mut rs = vec![];
    let mut visited = vec![0; D * D];
    let mut iter = 0;
    loop {
        rs.clear();
        for i in 0..D {
            for j in 0..D {
                if i + usize::abs_diff(j, D / 2) >= 2 {
                    rs.push((i, j));
                }
            }
        }
        rs.shuffle(&mut rng);
        rs.truncate(N);
        let mut cs = vec![!0; D * D];
        for &(i, j) in &rs {
            cs[i * D + j] = 0;
        }
        reachable(D, &cs, !0, !0, &mut visited, &mut iter);
        let mut ok = true;
        for i in 0..D {
            for j in 0..D {
                if cs[i * D + j] != 0 && visited[i * D + j] != iter {
                    ok = false;
                }
            }
        }
        if ok {
            break;
        }
    }
    let mut ts = (0..D * D - 1 - N).collect_vec();
    ts.shuffle(&mut rng);
    Input { D, rs, ts }
}

const DIJ: [(usize, usize); 4] = [(0, 1), (1, 0), (0, !0), (!0, 0)];

fn reachable(n: usize, cs: &Vec<usize>, ti: usize, tj: usize, visited: &mut Vec<usize>, iter: &mut usize) -> bool {
    if cs[n / 2] != !0 {
        return false;
    }
    *iter += 1;
    let mut stack = vec![(0, n / 2)];
    visited[n / 2] = *iter;
    while let Some((i, j)) = stack.pop() {
        if (i, j) == (ti, tj) {
            return true;
        }
        for (di, dj) in DIJ {
            let i2 = i + di;
            let j2 = j + dj;
            if i2 < n && j2 < n && cs[i2 * n + j2] == !0 && visited[i2 * n + j2].setmax(*iter) {
                stack.push((i2, j2));
            }
        }
    }
    false
}

pub fn compute_score(input: &Input, out: &[(usize, usize)]) -> (i64, String) {
    let (mut score, err, _) = compute_score_details(input, out);
    if err.len() > 0 {
        score = 0;
    }
    (score, err)
}

fn compute_score_details(
    input: &Input,
    out: &[(usize, usize)],
) -> (i64, String, (Vec<usize>, Vec<usize>, Option<(usize, usize)>)) {
    let mut cs = vec![!0; input.D * input.D];
    let mut visited = vec![0; input.D * input.D];
    let mut iter = 0;
    let mut num_put = 0;
    let mut order = vec![];
    let mut last = None;
    for &(i, j) in &input.rs {
        cs[i * input.D + j] = !1;
    }
    for &a in out {
        last = Some(a.clone());
        if num_put < input.D * input.D - 1 - input.rs.len() {
            if a == (0, input.D / 2) {
                return (
                    0,
                    format!("You cannot put containers on the entrance ({}, {})", 0, input.D / 2),
                    (cs, order, last),
                );
            } else if cs[a.0 * input.D + a.1] == !1 {
                return (0, format!("({}, {}) contains an obstacle", a.0, a.1), (cs, order, last));
            } else if cs[a.0 * input.D + a.1] != !0 {
                return (
                    0,
                    format!("({}, {}) already contains a container", a.0, a.1),
                    (cs, order, last),
                );
            } else if !reachable(input.D, &cs, a.0, a.1, &mut visited, &mut iter) {
                return (0, format!("({}, {}) is not reachalbe", a.0, a.1), (cs, order, last));
            }
            cs[a.0 * input.D + a.1] = input.ts[num_put];
            num_put += 1;
        } else {
            if cs[a.0 * input.D + a.1] == !0 || cs[a.0 * input.D + a.1] == !1 {
                return (
                    0,
                    format!("({}, {}) does not contain a container", a.0, a.1),
                    (cs, order, last),
                );
            }
            let c = cs[a.0 * input.D + a.1];
            cs[a.0 * input.D + a.1] = !0;
            if !reachable(input.D, &cs, a.0, a.1, &mut visited, &mut iter) {
                cs[a.0 * input.D + a.1] = c;
                return (0, format!("({}, {}) is not reachalbe", a.0, a.1), (cs, order, last));
            }
            order.push(c);
        }
    }
    let mut inv = 0;
    for i in 0..order.len() {
        for j in i + 1..order.len() {
            if order[i] > order[j] {
                inv += 1;
            }
        }
    }
    let K = (input.D * input.D - input.rs.len()) * (input.D * input.D - 1 - input.rs.len()) / 2;
    let score = (K - inv) as f64 / K as f64;
    let err = if order.len() < input.D * input.D - 1 - input.rs.len() {
        format!("Containers are still remaining.")
    } else {
        String::new()
    };
    ((1e9 * score).round() as i64, err, (cs, order, last))
}

/// 0 <= val <= 1
pub fn color(mut val: f64) -> String {
    val.setmin(1.0);
    val.setmax(0.0);
    let (r, g, b) = if val < 0.5 {
        let x = val * 2.0;
        (
            30. * (1.0 - x) + 144. * x,
            144. * (1.0 - x) + 255. * x,
            255. * (1.0 - x) + 30. * x,
        )
    } else {
        let x = val * 2.0 - 1.0;
        (
            144. * (1.0 - x) + 255. * x,
            255. * (1.0 - x) + 30. * x,
            30. * (1.0 - x) + 70. * x,
        )
    };
    format!("#{:02x}{:02x}{:02x}", r.round() as i32, g.round() as i32, b.round() as i32)
}

pub fn rect(x: usize, y: usize, w: usize, h: usize, fill: &str) -> Rectangle {
    Rectangle::new()
        .set("x", x)
        .set("y", y)
        .set("width", w)
        .set("height", h)
        .set("fill", fill)
}

pub fn vis_default(input: &Input, out: &Output) -> (i64, String, String) {
    vis(input, &out.actions, true)
}

pub fn vis(input: &Input, out: &[(usize, usize)], show_number: bool) -> (i64, String, String) {
    let W = 720 / input.D;
    let H0 = W / 2 * (input.D * input.D - 1 + 2 * input.D - 1) / (2 * input.D) + 5 + W / 2;
    let (score, err, (cs, order, last)) = compute_score_details(input, &out);
    let mut doc = svg::Document::new()
        .set("id", "vis")
        .set("viewBox", (-5, -5, W * input.D + 10, W * input.D + H0 + 10))
        .set("width", W * input.D + 10)
        .set("height", W * input.D + H0 + 10)
        .set("style", "background-color:white");
    doc = doc.add(Style::new(format!(
        "text {{text-anchor: middle;dominant-baseline: central;}}"
    )));
    doc = doc.add(
        Line::new()
            .set("x1", 0)
            .set("y1", H0)
            .set("x2", W * (input.D / 2))
            .set("y2", H0)
            .set("stroke", "black")
            .set("stroke-width", 3),
    );
    doc = doc.add(
        Line::new()
            .set("x1", W * (input.D / 2 + 1))
            .set("y1", H0)
            .set("x2", W * input.D)
            .set("y2", H0)
            .set("stroke", "black")
            .set("stroke-width", 3),
    );
    doc = doc.add(
        Line::new()
            .set("x1", 0)
            .set("y1", H0)
            .set("x2", 0)
            .set("y2", H0 + W * input.D)
            .set("stroke", "black")
            .set("stroke-width", 3),
    );
    doc = doc.add(
        Line::new()
            .set("x1", W * input.D)
            .set("y1", H0)
            .set("x2", W * input.D)
            .set("y2", H0 + W * input.D)
            .set("stroke", "black")
            .set("stroke-width", 3),
    );
    doc = doc.add(
        Line::new()
            .set("x1", 0)
            .set("y1", H0 + W * input.D)
            .set("x2", W * input.D)
            .set("y2", H0 + W * input.D)
            .set("stroke", "black")
            .set("stroke-width", 3),
    );
    let num_put = cs.iter().filter(|&&c| c != !0 && c != !1).count() + order.len();
    let mut used = vec![false; input.D * input.D - 1 - input.rs.len()];
    for i in 0..input.D {
        for j in 0..input.D {
            if cs[i * input.D + j] != !0 && cs[i * input.D + j] != !1 {
                used[cs[i * input.D + j]] = true;
            }
        }
    }
    if num_put < input.D * input.D - 1 - input.rs.len() {
        for i in 0..input.D * input.D - 1 - input.rs.len() {
            doc = doc.add(
                rect(
                    i % (2 * input.D) * (W / 2),
                    i / (2 * input.D) * (W / 2),
                    W / 2,
                    W / 2,
                    &if !used[i] {
                        color(i as f64 / (input.D * input.D - 1) as f64)
                    } else {
                        "lightgray".to_owned()
                    },
                )
                .set("stroke", "black")
                .set("stroke-width", 1),
            );
            if show_number {
                doc = doc.add(
                    svg::node::element::Text::new()
                        .set("x", i % (2 * input.D) * (W / 2) + W / 4)
                        .set("y", i / (2 * input.D) * (W / 2) + W / 4)
                        .set("font-size", W / 4)
                        .add(Text::new(i.to_string())),
                );
            }
        }
    }
    for i in 0..order.len() {
        doc = doc.add(
            rect(
                i % (2 * input.D) * (W / 2),
                i / (2 * input.D) * (W / 2),
                W / 2,
                W / 2,
                &color(order[i] as f64 / (input.D * input.D - 1) as f64),
            )
            .set("stroke", "black")
            .set("stroke-width", 1),
        );
        if show_number {
            doc = doc.add(
                svg::node::element::Text::new()
                    .set("x", i % (2 * input.D) * (W / 2) + W / 4)
                    .set("y", i / (2 * input.D) * (W / 2) + W / 4)
                    .set("font-size", W / 4)
                    .add(Text::new(order[i].to_string())),
            );
        }
    }
    for i in 0..input.D {
        for j in 0..input.D {
            let c = cs[i * input.D + j];
            if c == !0 {
                doc = doc.add(
                    Group::new()
                        .add(Title::new().add(Text::new(format!("({}, {})", i, j))))
                        .add(
                            rect(j * W, H0 + i * W, W, W, "#FFFFFFFF")
                                .set("stroke", "lightgray")
                                .set("stroke-width", 1),
                        )
                        .set("onclick", format!("manual({}, {})", i, j)),
                );
            } else if c == !1 {
                doc = doc.add(
                    Group::new()
                        .add(Title::new().add(Text::new(format!("({}, {})", i, j))))
                        .add(
                            rect(j * W, H0 + i * W, W, W, "lightgray")
                                .set("stroke", "lightgray")
                                .set("stroke-width", 1),
                        )
                        .set("onclick", format!("manual({}, {})", i, j)),
                );
            }
        }
    }
    for i in 0..input.D {
        for j in 0..input.D {
            let c = cs[i * input.D + j];
            if c != !0 && c != !1 {
                let mut g = Group::new().add(Title::new().add(Text::new(format!("({}, {})", i, j))));
                g = g.add(
                    rect(j * W, H0 + i * W, W, W, &color(c as f64 / (input.D * input.D - 1) as f64))
                        .set("stroke", "black")
                        .set("stroke-width", 1),
                );
                if show_number {
                    g = g.add(
                        svg::node::element::Text::new()
                            .set("x", j * W + W / 2)
                            .set("y", H0 + i * W + W / 2)
                            .set("font-size", W / 2)
                            .add(Text::new(c.to_string())),
                    );
                }
                doc = doc.add(g.set("onclick", format!("manual({}, {})", i, j)));
            }
        }
    }
    if let Some((i, j)) = last {
        doc = doc.add(
            rect(j * W, H0 + i * W, W, W, "none")
                .set("stroke", "black")
                .set("stroke-width", 5),
        );
    }
    if num_put < input.D * input.D - 1 - input.rs.len() {
        let i = input.ts[num_put];
        doc = doc.add(
            rect(i % (2 * input.D) * (W / 2), i / (2 * input.D) * (W / 2), W / 2, W / 2, "none")
                .set("stroke", "black")
                .set("stroke-width", 5),
        );
    }
    (score, err, doc.to_string())
}
