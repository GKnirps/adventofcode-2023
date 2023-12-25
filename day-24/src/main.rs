use std::cmp::Ordering;
use std::env;
use std::fs::read_to_string;
use std::ops::Range;
use std::path::Path;

mod rational;
use rational::Rational;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let hailstones = parse(&content)?;

    let n_2d_intersections =
        count_2d_crossings(&hailstones, 200000000000000.into(), 400000000000000.into());
    println!("{n_2d_intersections} two-dimensional intersections occur in the test area");

    if let Some(rock) = find_correct_2d_velocity(&hailstones, -500..501) {
        println!(
            "The sum of x, y and z of the rock's starting position is {}",
            rock.pos.0 + rock.pos.1 + rock.pos.2
        );
    } else {
        println!("This hail is a mess! How am I supposed to throw a rock in this mess?");
    }

    Ok(())
}

fn hits(rock: &Hailstone, hail: &Hailstone) -> bool {
    if rock == hail {
        return true;
    }
    let Hailstone {
        pos: (sx, sy, sz),
        vel: (vx, vy, vz),
    } = *rock;
    let Hailstone {
        pos: (sax, say, saz),
        vel: (vax, vay, vaz),
    } = *hail;
    let t = if vx != vax {
        (sax - sx) / (vx - vax)
    } else if sx != sax {
        return false;
    } else if vy != vay {
        (say - sy) / (vy - vay)
    } else if sy != say {
        return false;
    } else if vz != vaz {
        (saz - sz) / (vz - vaz)
    } else {
        return false;
    };

    sx + t * vx == sax + t * vax && sy + t * vy == say + t * vay && sz + t * vz == saz + t * vaz
}

fn compress_ranges(mut ranges: Vec<(Rational, Rational)>) -> Vec<(Rational, Rational)> {
    if ranges.is_empty() {
        return vec![];
    }
    ranges.sort_unstable_by_key(|(from, _)| *from);
    let mut out: Vec<(Rational, Rational)> = Vec::with_capacity(ranges.len());
    let mut current: (Rational, Rational) = ranges[0];
    for (from, to) in &ranges[1..] {
        if *from <= current.1 + 1.into() {
            current.1 = current.1.max(*to);
        } else {
            out.push(current);
            current = (*from, *to);
        }
    }
    out.push(current);
    out
}

fn bin_search_ranges(ranges: &[(Rational, Rational)], needle: Rational) -> bool {
    ranges
        .binary_search_by(|(from, to)| {
            if *from <= needle {
                if needle < *to {
                    Ordering::Equal
                } else {
                    Ordering::Greater
                }
            } else {
                Ordering::Less
            }
        })
        .is_ok()
}

fn find_correct_2d_velocity(
    hailstones: &[Hailstone],
    brute_force_range: Range<i128>,
) -> Option<Hailstone> {
    // I'm tired of trying to find an analytical solution, let's brute force it.
    if hailstones.len() < 2 {
        return None;
    }
    let ignore_x_ranges: Vec<(Rational, Rational)> = compress_ranges(
        hailstones
            .iter()
            .enumerate()
            .flat_map(|(i, hs1)| {
                hailstones[i + 1..].iter().filter_map(|hs2| {
                    let p1 = hs1.pos.0;
                    let v1 = hs1.vel.0;
                    let p2 = hs2.pos.0;
                    let v2 = hs2.vel.0;
                    if p1 > p2 && v1 > v2 || p2 > p1 && v2 > v1 {
                        Some((v1.min(v2), v1.max(v2)))
                    } else {
                        None
                    }
                })
            })
            .collect(),
    );
    let ignore_y_ranges: Vec<(Rational, Rational)> = compress_ranges(
        hailstones
            .iter()
            .enumerate()
            .flat_map(|(i, hs1)| {
                hailstones[i + 1..].iter().filter_map(|hs2| {
                    let p1 = hs1.pos.1;
                    let v1 = hs1.vel.1;
                    let p2 = hs2.pos.1;
                    let v2 = hs2.vel.1;
                    if p1 > p2 && v1 > v2 || p2 > p1 && v2 > p1 {
                        Some((v1.min(v2), v1.max(v2)))
                    } else {
                        None
                    }
                })
            })
            .collect(),
    );
    let ignore_z_ranges: Vec<(Rational, Rational)> = compress_ranges(
        hailstones
            .iter()
            .enumerate()
            .flat_map(|(i, hs1)| {
                hailstones[i + 1..].iter().filter_map(|hs2| {
                    let p1 = hs1.pos.2;
                    let v1 = hs1.vel.2;
                    let p2 = hs2.pos.2;
                    let v2 = hs2.vel.2;
                    if p1 > p2 && v1 > v2 || p2 > p1 && v2 > p1 {
                        Some((v1.min(v2), v1.max(v2)))
                    } else {
                        None
                    }
                })
            })
            .collect(),
    );
    for vx in brute_force_range.clone() {
        let vx: Rational = vx.into();
        if bin_search_ranges(&ignore_x_ranges, vx) {
            continue;
        }
        for vy in brute_force_range.clone() {
            let vy: Rational = vy.into();
            if bin_search_ranges(&ignore_y_ranges, vy) {
                continue;
            }
            let hs0 = Hailstone {
                pos: hailstones[0].pos,
                vel: (
                    hailstones[0].vel.0 - vx,
                    hailstones[0].vel.1 - vy,
                    hailstones[0].vel.2,
                ),
            };
            if let Some((x, y)) = cross_2d(
                &hs0,
                &Hailstone {
                    pos: hailstones[1].pos,
                    vel: (
                        hailstones[1].vel.0 - vx,
                        hailstones[1].vel.1 - vy,
                        hailstones[1].vel.2,
                    ),
                },
            ) {
                if hailstones[2..].iter().all(|hs2| {
                    cross_2d(
                        &hs0,
                        &Hailstone {
                            pos: hs2.pos,
                            vel: (hs2.vel.0 - vx, hs2.vel.1 - vy, hs2.vel.2),
                        },
                    ) == Some((x, y))
                }) {
                    for vz in brute_force_range.clone() {
                        let vz: Rational = vz.into();
                        if bin_search_ranges(&ignore_z_ranges, vz) {
                            continue;
                        }
                        // FIXME: this may not work for all hailstones, velocity on the x axis may
                        // be 0
                        let t = (x - hs0.pos.0) / hs0.vel.0;
                        let vel = (vx, vy, vz);
                        let hs0 = &hailstones[0];
                        let pos = (
                            hs0.pos.0 + t * hs0.vel.0 - t * vx,
                            hs0.pos.1 + t * hs0.vel.1 - t * vy,
                            hs0.pos.2 + t * hs0.vel.2 - t * vz,
                        );
                        let rock = Hailstone { pos, vel };
                        if hailstones.iter().all(|hailstone| hits(&rock, hailstone)) {
                            return Some(rock);
                        }
                    }
                }
            }
        }
    }
    None
}

fn count_2d_crossings(hailstones: &[Hailstone], from: Rational, to: Rational) -> usize {
    hailstones
        .iter()
        .enumerate()
        .map(|(i, hs1)| {
            hailstones[i + 1..]
                .iter()
                .filter(|hs2| {
                    if let Some((cross_x, cross_y)) = cross_2d(hs1, hs2) {
                        cross_x >= from && cross_y >= from && cross_x <= to && cross_y <= to
                    } else {
                        false
                    }
                })
                .count()
        })
        .sum()
}

// check for cross between hailstones trajectories, ignoring the z axis
// returns None if the run parallel or Some((x, y)) if the crossing is in the future of both
// trajectories
fn cross_2d(hs1: &Hailstone, hs2: &Hailstone) -> Option<(Rational, Rational)> {
    let (a1x, a1y, _) = hs1.pos;
    let (b1x, b1y, _) = hs1.vel;
    let (a2x, a2y, _) = hs2.pos;
    let (b2x, b2y, _) = hs2.vel;
    if a1x == a2x && a1y == a2y {
        Some((a1x, a2x))
    } else if b2x != 0 {
        let denom = -b1y + (b1x * b2y) / b2x;
        if denom == 0 {
            None
        } else {
            let m = (a1y - a2y - (a1x - a2x) / b2x * b2y) / denom;
            let n = (a1x - a2x + m * b1x) / b2x;
            if m < 0.into() || n < 0.into() {
                None
            } else {
                Some((a1x + m * b1x, a1y + m * b1y))
            }
        }
    } else if b2y != 0 {
        let denom = -b1x + (b1y * b2x) / b2y;
        if denom == 0 {
            None
        } else {
            let m = (a1x - a2x - (a1y - a2y) / b2y * b2x) / denom;
            let n = (a1y - a2y + m * b1y) / b2y;
            if m < 0.into() || n < 0.into() {
                None
            } else {
                Some((a1x + m * b1x, a1y + m * b1y))
            }
        }
    } else {
        None
    }
}

fn parse(input: &str) -> Result<Vec<Hailstone>, String> {
    input.lines().map(parse_hailstone).collect()
}

fn parse_hailstone(line: &str) -> Result<Hailstone, String> {
    let (pos, vel) = line
        .split_once(" @ ")
        .ok_or_else(|| format!("unable to split position from velocity in line '{line}'"))?;
    let pos = parse_triplet(pos)?;
    let vel = parse_triplet(vel)?;
    if vel.0 == 0 && vel.1 == 0 {
        Err(format!("Hailstone '{line}' has velocity (0, 0, z)"))
    } else {
        Ok(Hailstone { pos, vel })
    }
}

fn parse_triplet(s: &str) -> Result<(Rational, Rational, Rational), String> {
    let mut parts = s.splitn(3, ", ").map(|n| {
        n.trim()
            .parse::<i128>()
            .map_err(|e| format!("unable to parse '{n}' in triplet '{s}': {e}"))
    });
    Ok((
        Rational::new(
            parts
                .next()
                .ok_or_else(|| format!("expected exactly three numbers in triplet '{s}'"))??,
            1,
        ),
        Rational::new(
            parts
                .next()
                .ok_or_else(|| format!("expected exactly three numbers in triplet '{s}'"))??,
            1,
        ),
        Rational::new(
            parts
                .next()
                .ok_or_else(|| format!("expected exactly three numbers in triplet '{s}'"))??,
            1,
        ),
    ))
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct Hailstone {
    pos: (Rational, Rational, Rational),
    vel: (Rational, Rational, Rational),
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"19, 13, 30 @ -2,  1, -2
18, 19, 22 @ -1, -1, -2
20, 25, 34 @ -2, -2, -4
12, 31, 28 @ -1, -2, -1
20, 19, 15 @  1, -5, -3
"#;

    #[test]
    fn count_2d_crossings_works_for_example() {
        // given
        let hailstones = parse(EXAMPLE).expect("expected successful parsing");

        // when
        let n = count_2d_crossings(&hailstones, 7.into(), 27.into());

        // then
        assert_eq!(n, 2);
    }

    #[test]
    fn hits_checks_hits_for_example() {
        // given
        let hailstones = parse(EXAMPLE).expect("expected successful parsing");
        assert_eq!(hailstones.len(), 5);
        let example_rock = Hailstone {
            pos: (24.into(), 13.into(), 10.into()),
            vel: ((-3).into(), 1.into(), 2.into()),
        };

        // when/then
        assert!(hits(&example_rock, &hailstones[0]));
        assert!(hits(&example_rock, &hailstones[1]));
        assert!(hits(&example_rock, &hailstones[2]));
        assert!(hits(&example_rock, &hailstones[3]));
        assert!(hits(&example_rock, &hailstones[4]));
    }

    #[test]
    fn find_correct_2d_velocity_finds_candidates_for_example() {
        // given
        let hailstones = parse(EXAMPLE).expect("expected successful parsing");

        // when
        let rock = find_correct_2d_velocity(&hailstones, -20..21);

        // then
        assert_eq!(
            rock,
            Some(Hailstone {
                pos: (24.into(), 13.into(), 10.into()),
                vel: ((-3).into(), 1.into(), 2.into())
            })
        )
    }
}
