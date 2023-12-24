use std::env;
use std::fs::read_to_string;
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

    Ok(())
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
}
