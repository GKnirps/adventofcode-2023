use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;

    let stars = parse(&content)?;

    let adjusted_stars = adjust_space(stars);
    let distances = distance_sum(&adjusted_stars);
    println!("The sum of distances between stars is {distances}");

    Ok(())
}

type Pos = (usize, usize);

fn parse(input: &str) -> Result<Vec<Pos>, String> {
    // assumption: input is rectangular, all lines end in "\n"
    let width = input
        .lines()
        .next()
        .ok_or_else(|| "input contains no lines".to_owned())?
        .len()
        + 1;

    Ok(input
        .bytes()
        .enumerate()
        .filter(|(_, c)| *c == b'#')
        .map(|(i, _)| (i % width, i / width))
        .collect())
}

fn adjust_space(mut stars: Vec<Pos>) -> Vec<Pos> {
    if stars.len() < 2 {
        return stars;
    }

    stars.sort_unstable_by_key(|(x, _)| *x);
    let mut empty_rows: Vec<(usize, usize)> = Vec::with_capacity(stars.len() * 10);
    for i in 0..(stars.len() - 1) {
        let x1 = stars[i].0;
        let x2 = stars[i + 1].0;
        if x2 - x1 > 1 {
            empty_rows.push((i, x2 - x1));
        }
    }
    for (i, star) in stars.iter_mut().enumerate() {
        for (j, diff) in &empty_rows {
            if i > *j {
                star.0 += diff - 1;
            }
        }
    }

    stars.sort_unstable_by_key(|(_, y)| *y);
    empty_rows.clear();
    let mut empty_cols: Vec<(usize, usize)> = empty_rows;
    for i in 0..(stars.len() - 1) {
        let y1 = stars[i].1;
        let y2 = stars[i + 1].1;
        if y2 - y1 > 1 {
            empty_cols.push((i, y2 - y1));
        }
    }
    for (i, star) in stars.iter_mut().enumerate() {
        for (j, diff) in &empty_cols {
            if i > *j {
                star.1 += diff - 1;
            }
        }
    }

    stars
}

fn distance_sum(stars: &[Pos]) -> usize {
    stars
        .iter()
        .enumerate()
        .map(|(i, (x1, y1))| {
            stars
                .iter()
                .skip(i + 1)
                .map(|(x2, y2)| x1.max(x2) - x1.min(x2) + y1.max(y2) - y1.min(y2))
                .sum::<usize>()
        })
        .sum()
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"...#......
.......#..
#.........
..........
......#...
.#........
.........#
..........
.......#..
#...#.....
"#;

    #[test]
    fn distance_sum_works_for_example() {
        // given
        let stars = parse(EXAMPLE).expect("expected successful parsing");

        // when
        let stars = adjust_space(stars);
        let distances = distance_sum(&stars);

        // then
        assert_eq!(distances, 374);
    }
}
