use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let bricks = parse(&content)?;

    let one_disintegrate_options = safely_disintegratable(&bricks);
    println!(
        "{one_disintegrate_options} bricks could be safely chosen as the one to get disintegrated."
    );

    let chain_reaction_fallen = sum_chain_reaction(&bricks);
    println!("{chain_reaction_fallen} other bricks fell in all chain reactions.");

    Ok(())
}

// preconditions: bricks must be sorted by lowest z
fn safely_disintegratable(bricks: &[Brick]) -> usize {
    let supported_by = find_supported_by(bricks);
    let mut removable = vec![true; bricks.len()];
    for supports in supported_by.values() {
        if supports.len() == 1 {
            removable[supports[0]] = false;
        }
    }
    removable.iter().filter(|r| **r).count()
}

fn find_supported_by(bricks: &[Brick]) -> HashMap<usize, Vec<usize>> {
    let settled = settle_bricks(bricks);
    let mut supported_by: HashMap<usize, Vec<usize>> = HashMap::with_capacity(bricks.len());
    for (i, brick) in settled.iter().rev().enumerate() {
        for (j, possible_support) in settled.iter().rev().enumerate().skip(i + 1) {
            if brick.from.2 == possible_support.to.2 + 1 && overlap_xy(brick, possible_support) {
                supported_by
                    .entry(i)
                    .or_insert(Vec::with_capacity(8))
                    .push(j);
            }
        }
    }
    supported_by
}

fn sum_chain_reaction(bricks: &[Brick]) -> usize {
    let supported_by = find_supported_by(bricks);
    println!(
        "supported bricks: {}, total: {}",
        supported_by.len(),
        bricks.len()
    );

    (0..bricks.len())
        .map(|brick_i| chain_reaction(brick_i, &supported_by))
        .sum()
}

fn chain_reaction(removed_brick: usize, supported_by: &HashMap<usize, Vec<usize>>) -> usize {
    let mut fallen: HashSet<usize> = HashSet::with_capacity(supported_by.len());
    fallen.insert(removed_brick);

    let mut can_fall: HashSet<usize> = supported_by.keys().copied().collect();
    let mut newly_fallen: Vec<usize> = Vec::with_capacity(can_fall.len());
    newly_fallen.push(removed_brick);
    // there is probably a more efficient way than to go over all the bricks over and over again,
    // but it should still work
    while !newly_fallen.is_empty() {
        for brick_i in newly_fallen.drain(..) {
            can_fall.remove(&brick_i);
        }
        for candidate in &can_fall {
            if fallen.contains(candidate) {
                continue;
            }
            if supported_by
                .get(candidate)
                .map(|supports| supports.iter().all(|support| fallen.contains(support)))
                .unwrap_or(false)
            {
                fallen.insert(*candidate);
                newly_fallen.push(*candidate);
            }
        }
    }

    fallen.len() - 1
}

fn settle_bricks(bricks: &[Brick]) -> Vec<Brick> {
    let mut settled = Vec::with_capacity(bricks.len());
    for brick in bricks {
        let mut next = brick.clone();
        next.to.2 = next.to.2 + 1 - next.from.2;
        next.from.2 = 1;
        for s in settled.iter().rev() {
            if overlap_xy(s, &next) {
                next.to.2 -= next.from.2;
                next.from.2 = s.to.2 + 1;
                next.to.2 += next.from.2;
                break;
            }
        }
        settled.push(next);
        // TODO: inefficient, but for now it works fast enough
        settled.sort_by_key(|brick| brick.to.2);
    }
    settled
}

fn overlap_xy(b1: &Brick, b2: &Brick) -> bool {
    (b1.from.0 <= b2.from.0 && b1.to.0 >= b2.from.0
        || b1.from.0 >= b2.from.0 && b1.from.0 <= b2.to.0)
        && (b1.from.1 <= b2.from.1 && b1.to.1 >= b2.from.1
            || b1.from.1 >= b2.from.1 && b1.from.1 <= b2.to.1)
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct Brick {
    from: (u32, u32, u32),
    to: (u32, u32, u32),
}

fn parse(input: &str) -> Result<Vec<Brick>, String> {
    let mut bricks: Vec<Brick> = input.lines().map(parse_brick).collect::<Result<_, _>>()?;
    bricks.sort_by_key(|b| b.from.2);
    Ok(bricks)
}

fn parse_brick(line: &str) -> Result<Brick, String> {
    let (from, to) = line
        .split_once('~')
        .ok_or_else(|| "unable to split brick ends from each other".to_string())?;
    let from = parse_coords(from)?;
    let to = parse_coords(to)?;
    if from.0 > to.0 || from.1 > to.1 || from.2 > to.2 {
        // the inputs are ordered this way. If they are not, we could order them here instead of
        // returning an error
        Err(format!(
            "cube '{line}' has to-values that are higher than from-values"
        ))
    } else {
        Ok(Brick { from, to })
    }
}

fn parse_coords(s: &str) -> Result<(u32, u32, u32), String> {
    let mut coords = s.splitn(3, ',').map(|part| {
        part.parse::<u32>()
            .map_err(|e| format!("unable to parse ordinate '{part}': {e}"))
    });
    Ok((
        coords
            .next()
            .ok_or_else(|| "expected three coordinates".to_string())??,
        coords
            .next()
            .ok_or_else(|| "expected three coordinates".to_string())??,
        coords
            .next()
            .ok_or_else(|| "expected three coordinates".to_string())??,
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"1,0,1~1,2,1
0,0,2~2,0,2
0,2,3~2,2,3
0,0,4~0,2,4
2,0,5~2,2,5
0,1,6~2,1,6
1,1,8~1,1,9
"#;

    #[test]
    fn safely_disintegratable_works_for_example() {
        // given
        let bricks = parse(EXAMPLE).expect("expected successful parsing");

        // when
        let n = safely_disintegratable(&bricks);

        // then
        assert_eq!(n, 5);
    }

    #[test]
    fn sum_chain_reaction_works_for_example() {
        // given
        let bricks = parse(EXAMPLE).expect("expected successful parsing");

        // when
        let n = sum_chain_reaction(&bricks);

        // then
        assert_eq!(n, 7);
    }
}
