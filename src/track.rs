use std::fs::File;
use std::io::{BufRead, BufReader};
use image::{ImageBuffer, Rgba};

use crate::vectors::Vec2;

const HALF_PI: f32 = std::f32::consts::PI / 2.0;
const PI: f32 = std::f32::consts::PI;
const TAU: f32 = std::f32::consts::PI * 2.0;

pub struct Track {
    pub radius: f32,
    pub start_finish: LineSegment,
    pub segments: Vec<LineSegment>,
    pub length: f32,
}

impl Track {
    pub fn validate(&self) -> bool {
        if !connects(self.start_finish.get_end(), self.segments[0].get_start()) {
            return false
        }
        for i in 0..(self.segments.len() - 1) {
            if !connects(self.segments[i].get_end(), self.segments[i + 1].get_start()) {
                return false
            }
        }
        if !connects(self.segments[self.segments.len() - 1].get_end(), self.start_finish.get_start()) {
            return false
        }
        true
    }
    pub fn from_file(path: &str) -> Self {
        let file = File::open(path).unwrap();
        let mut lines = BufReader::new(file).lines();

        let first_line = lines.next().unwrap().unwrap();
        let radius = first_line.parse::<f32>().unwrap();

        let second_line = lines.next().unwrap().unwrap();
        let mut second_line_words = second_line.split(" ");

        let start = Vec2 { x: next(&mut second_line_words), y: next(&mut second_line_words)};
        let end =  Vec2 { x: next(&mut second_line_words), y: next(&mut second_line_words)};
        let start_finish_length = (end - start).length();
        let start_finish = LineSegment::Streight( StreightLine {
            start,
            end,
            length: start_finish_length,
        });
        let mut total_length = start_finish_length;
        let mut segments = Vec::new();
        for line_res in lines {
            let line = line_res.unwrap();
            let mut line_words = line.split(" ");
            match line_words.next().unwrap() {
                "line" => {
                    let start = Vec2 { x: next(&mut line_words), y: next(&mut line_words)};
                    let end =  Vec2 { x: next(&mut line_words), y: next(&mut line_words)};
                    let length = (end - start).length();
                    segments.push(LineSegment::Streight( StreightLine {
                        start,
                        end,
                        length,
                    }));
                    total_length += length;
                }
                "radial" => {
                    let centre = Vec2 { x: next(&mut line_words), y: next(&mut line_words)};
                    let radius = next(&mut line_words);
                    let start_bearing = next(&mut line_words);
                    let size = next(&mut line_words);
                    let length = size.abs() * radius;
                    segments.push(LineSegment::Radial( RadialLine {
                        centre,
                        radius,
                        start_bearing,
                        size,
                        length,
                    }));
                    total_length += length;
                }
                _ => { }
            }
        }
        Track { radius, start_finish, segments, length: total_length }
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
        if self.start_finish.contains(point, self.radius) {
            return self.start_finish.get_progression(point) * self.start_finish.get_length() / self.length;
        }
        let mut distance = self.start_finish.get_length();

        for segment in &self.segments {
            if segment.contains(point, self.radius) {
                return (distance + segment.get_progression(point) * segment.get_length()) / self.length;
            }
            distance += segment.get_length();
        }

        panic!("point given was not on the track")
    }
}
fn flip_vec2(input: Vec2) -> Vec2 {
    Vec2 { x: input.x, y: 1080.0 - input.y}
}
fn next(line_words: &mut std::str::Split<'_, &str>) -> f32 {
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
    fn get_start(&self) -> LineEnding {
        match self {
            LineSegment::Streight(line) => {
                LineEnding {
                    position: line.start,
                    bearing: (line.start - line.end).bearing(),
                }
            },
            LineSegment::Radial(line) => {
                LineEnding {
                    position: line.centre + Vec2::from_bearing(line.start_bearing) * line.radius,
                    bearing: line.start_bearing + -HALF_PI * line.size.signum(),                    
                }
            }     
        }
    }
    fn get_end(&self) -> LineEnding {
        match self {
            LineSegment::Streight(line) => {
                LineEnding {
                    position: line.end,
                    bearing: (line.end - line.start).bearing(),
                }
            },
            LineSegment::Radial(line) => {
                let bearing = line.start_bearing + line.size;
                LineEnding {
                    position: line.centre + Vec2::from_bearing(bearing) * line.radius,
                    bearing: bearing + HALF_PI * line.size.signum(),                    
                }
            },
        }    
    }
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
    fn contains(&self, point: Vec2, radius: f32) -> bool {
        match self {
            LineSegment::Streight(line) => {
                let dir = line.end - line.start;
                let len_sq = dir.lengthsq();
                if len_sq == 0.0 {
                    return (point - line.start).lengthsq() <= radius * radius;
                }
                let rel = point - line.start;
                let t = (rel.x * dir.x + rel.y * dir.y) / len_sq;
                if !(0.0..=1.0).contains(&t) {
                    return false;
                }

                let closest = line.start + dir * t;
                (point - closest).lengthsq() <= radius * radius
            },
            LineSegment::Radial(line) => {
                let offset = point - line.centre;

                if (offset.length() - line.radius).abs() > radius {
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
}

struct LineEnding {
    position: Vec2,
    bearing: f32,
}

fn connects(a: LineEnding, b: LineEnding) -> bool {
    let diff = (a.bearing - b.bearing).rem_euclid(TAU);
    (a.position - b.position).lengthsq() < 1.0 && (diff - PI).abs() < HALF_PI / 8.0
}