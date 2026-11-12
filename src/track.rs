use std::fs::File;
use std::io::{BufRead, BufReader};
use image::{ImageBuffer, Rgba};

use crate::vectors::*;

const HALF_PI: f32 = std::f32::consts::PI / 2.0;
const TAU: f32 = std::f32::consts::PI * 2.0;

const MAX_RAY_HOPS: usize = 64;
const RAY_EPSILON: f32 = 1e-3;

pub struct Track {
    pub radius: f32,
    pub start_finish: LineSegment,
    pub segments: Vec<LineSegment>,
    pub length: f32,
    pub start_position: f32,
}

impl Track {
    pub fn from_file(path: &str) -> Self {
        let file = File::open(path).unwrap();
        let mut lines = BufReader::new(file).lines();

        let first_line = lines.next().unwrap().unwrap();
        let radius = first_line.parse::<f32>().unwrap();

        let second_line = lines.next().unwrap().unwrap();
        let start_position = second_line.parse::<f32>().unwrap();

        let mut total_length = 0.0;

        let third_line = lines.next().unwrap().unwrap();
        let start_finish = read_line(&mut total_length, third_line);

        let mut segments = Vec::new();
        for line_res in lines {
            let line = line_res.unwrap();
            segments.push(read_line(&mut total_length, line));
        }
        Track { radius, start_finish, segments, length: total_length, start_position }
    }
    pub fn render(&self, to: &str) {
        let mut image = ImageBuffer::from_pixel(1920, 1080, Rgba([0u8, 0, 0, 255]));

        let mut a = None;
        let mut b = None;
        self.draw_segment(&mut image, &mut a, &mut b, &self.start_finish);
        for segment in &self.segments {
            self.draw_segment(&mut image, &mut a, &mut b, segment);
        }
        image.save(to).unwrap()
    }
    fn draw_segment(&self, image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, a: &mut Option<Vec2>, b: &mut Option<Vec2>, segment: &LineSegment) {
        for i in 0..=4096 {
            let point = segment.get_point(i as f32 / 4096.0);
            let bearing = segment.get_bearing(i as f32 / 4096.0);
    
            let c = point + Vec2::from_bearing(bearing + HALF_PI) * self.radius;
            let d = point + Vec2::from_bearing(bearing - HALF_PI) * self.radius;
    
            if a.is_some() && b.is_some() {
                draw_polygon(image, vec![flip_vec2(a.unwrap()), flip_vec2(b.unwrap()), flip_vec2(c), flip_vec2(d)])
            }
    
            *a = Some(d);
            *b = Some(c);
        }
    }
    pub fn contains(&self, point: Vec2) -> bool {
        if self.start_finish.contains(point, self.radius) {
            return true;
        }   
        for segment in &self.segments {
            if segment.contains(point, self.radius) {
                return true;
            }   
        }
        false
    }
    pub fn get_progression(&self, point: Vec2) -> f32 {
        let start_offset = self.start_finish.get_length() * self.start_position;
        if self.start_finish.contains(point, self.radius) {
            return ((self.start_finish.get_progression(point) * self.start_finish.get_length() - start_offset) / self.length).rem_euclid(1.0);
        }
        let mut distance = self.start_finish.get_length();

        for segment in &self.segments {
            if segment.contains(point, self.radius) {
                return ((distance + segment.get_progression(point) * segment.get_length() - start_offset) / self.length).rem_euclid(1.0);
            }
            distance += segment.get_length();
        }




        panic!("point given was not on the track")
    }
    pub fn collide_ray(&self, origin: Vec2, direction: Vec2) -> Option<f32> {
        let mut travelled = 0.0;
        for _ in 0..MAX_RAY_HOPS {
            let point = origin + direction * travelled;
            let exit = std::iter::once(&self.start_finish)
                .chain(&self.segments)
                .find_map(|segment| segment.collide_ray(point, direction, self.radius));
            match exit {
                Some(distance) => travelled += distance + RAY_EPSILON,
                None => return (travelled > 0.0).then_some(travelled),
            }
        }
        Some(travelled)
    }
    pub fn get_start(&self) -> Transform {
        Transform { 
            position: self.start_finish.get_point(self.start_position),
            bearing: self.start_finish.get_bearing(self.start_position),
        }
    }
}

fn read_line(total_length: &mut f32, line: String) -> LineSegment {
    let mut line_words = line.split(" ");
    match line_words.next().unwrap() {
        "line" => {
            let start = next_vec2(&mut line_words);
            let end =  next_vec2(&mut line_words);
            let length = (end - start).length();
            *total_length += length;
            LineSegment::Streight( StreightLine {
                start,
                end,
                length,
            })
        }
        "radial" => {
            let centre = next_vec2(&mut line_words);
            let radius = next_f32(&mut line_words);
            let start_bearing = next_f32(&mut line_words);
            let size = next_f32(&mut line_words);
            let length = size.abs() * radius;
            *total_length += length;
            LineSegment::Radial( RadialLine {
                centre,
                radius,
                start_bearing,
                size,
                length,
            })
        }
        _ => { panic!("unregonised line type") }
    }
}

fn next_vec2(line_words: &mut std::str::Split<'_, &str>) -> Vec2 {
    Vec2 { x: next_f32(line_words), y: next_f32(line_words)}
}

fn next_f32(line_words: &mut std::str::Split<'_, &str>) -> f32 {
    line_words.next().unwrap().parse::<f32>().unwrap()
}

fn draw_polygon(image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, points: Vec<Vec2>,)
{
    if points.len() < 3 {
        return;
    }

    let width = image.width() as i32;
    let height = image.height() as i32;

    let min_y = points.iter().map(|p| p.y as i32).min().unwrap().max(0);

    let max_y = points.iter().map(|p| p.y as i32).max().unwrap().min(height - 1);

    for y in min_y..=max_y {
        let mut intersections = Vec::new();

        for i in 0..points.len() {
            let p1 = points[i];
            let p2 = points[(i + 1) % points.len()];

            if (p1.y <= y as f32 && p2.y > y as f32)
                || (p2.y <= y as f32 && p1.y > y as f32)
            {
                let t = (y as f32 - p1.y) / (p2.y - p1.y);
                let x = p1.x + t * (p2.x - p1.x);

                intersections.push(x);
            }
        }

        intersections.sort_by(|a, b| a.partial_cmp(b).unwrap());

        for pair in intersections.chunks_exact(2) {
            let x_start = (pair[0] as i32).max(0);
            let x_end = (pair[1] as i32).min(width - 1);

            for x in x_start..=x_end {
                image.put_pixel(
                    x as u32,
                    y as u32,
                    Rgba([127, 127, 127, 127]),
                );
            }
        }
    }
}
pub enum LineSegment {
    Streight(StreightLine),
    Radial(RadialLine),
}

pub struct StreightLine {
    start: Vec2,
    end: Vec2,
    length: f32,
}

pub struct RadialLine {
    centre: Vec2,
    radius: f32,
    start_bearing: f32,
    size: f32,
    length: f32,
}

impl LineSegment {
    pub fn get_point(&self, t: f32) -> Vec2 {
        match self {
            LineSegment::Streight(line) => {
                line.start.interpolate(line.end, t)
            },
            LineSegment::Radial(line) => {
                line.centre + Vec2::from_bearing(line.start_bearing + line.size * t) * line.radius
            },
        }    
    }
    pub fn get_bearing(&self, t: f32) -> f32 {
        match self {
            LineSegment::Streight(line) => {
                (line.end - line.start).bearing()
            },
            LineSegment::Radial(line) => {
                line.start_bearing + line.size * t + HALF_PI * line.size.signum()
            },
        } 
    }
    fn contains(&self, point: Vec2, track_radius: f32) -> bool {
        match self {
            LineSegment::Streight(line) => {
                let dir = line.end - line.start;
                let len_sq = dir.lengthsq();
                if len_sq == 0.0 {
                    return (point - line.start).lengthsq() <= track_radius * track_radius;
                }
                let rel = point - line.start;
                let t = (rel.x * dir.x + rel.y * dir.y) / len_sq;
                if !(0.0..=1.0).contains(&t) {
                    return false;
                }

                let closest = line.start + dir * t;
                (point - closest).lengthsq() <= track_radius * track_radius
            },
            LineSegment::Radial(line) => {
                let offset = point - line.centre;

                if (offset.length() - line.radius).abs() > track_radius {
                    return false;
                }
                let relative_bearing =
                    ((offset.bearing() - line.start_bearing) * line.size.signum()).rem_euclid(TAU);
                relative_bearing <= line.size.abs()
            },
        } 
    }
    fn get_progression(&self, point: Vec2) -> f32 {
        match self {
            LineSegment::Streight(line) => {
                let dir = line.end - line.start;
                let len_sq = dir.lengthsq();
                if len_sq == 0.0 {
                    return 0.0;
                }
                let rel = point - line.start;
                (rel.x * dir.x + rel.y * dir.y) / len_sq
            },
            LineSegment::Radial(line) => {
                let arc = line.size.abs();
                if arc == 0.0 {
                    return 0.0;
                }
                let offset = point - line.centre;
                let relative = ((offset.bearing() - line.start_bearing) * line.size.signum()).rem_euclid(TAU);

                relative / arc
            },
        }
    }
    fn get_length(&self) -> f32 {
        match  self {
            LineSegment::Streight(line) => {
                line.length
            },
            LineSegment::Radial(line) => {
                line.length
            },   
        }
    }
fn collide_ray(&self, origin: Vec2, direction: Vec2, track_radius: f32) -> Option<f32> {
    if !self.contains(origin, track_radius) {
        return None;
    }
    match self {
        LineSegment::Streight(line) => {
            if line.length == 0.0 {
                return None;
            }
            let along = (line.end - line.start) / line.length;
            let across = Vec2 { x: -along.y, y: along.x };
            let rel = origin - line.start;

            let t_along = slab_exit(rel.dot(along), direction.dot(along), 0.0, line.length);
            let t_across = slab_exit(rel.dot(across), direction.dot(across), -track_radius, track_radius);
            Some(t_along.min(t_across))
        }
        LineSegment::Radial(line) => {
            let q = origin - line.centre;
            let b = q.dot(direction);
            let q_sq = q.lengthsq();
            let mut exit = f32::INFINITY;

            let outer = line.radius + track_radius;
            exit = exit.min(-b + (b * b - (q_sq - outer * outer)).max(0.0).sqrt());

            let inner = line.radius - track_radius;
            let disc = b * b - (q_sq - inner * inner);
            if inner > 0.0 && b < 0.0 && disc >= 0.0 {
                exit = exit.min(-b - disc.sqrt());
            }

            for edge in [line.start_bearing, line.start_bearing + line.size] {
                let edge_dir = Vec2::from_bearing(edge);
                let denom = direction.cross(edge_dir);
                if denom.abs() < f32::EPSILON {
                    continue;
                }
                let t = -q.cross(edge_dir) / denom;
                if t > 0.0 && (q + direction * t).dot(edge_dir) >= 0.0 {
                    exit = exit.min(t);
                }
            }
            Some(exit)
        }
    }
}
}


fn flip_vec2(input: Vec2) -> Vec2 {
    Vec2 { x: input.x, y: 1080.0 - input.y}
}


fn slab_exit(pos: f32, vel: f32, lo: f32, hi: f32) -> f32 {
    if vel > 0.0 {
        (hi - pos) / vel
    } else if vel < 0.0 {
        (lo - pos) / vel
    } else {
        f32::INFINITY
    }
}