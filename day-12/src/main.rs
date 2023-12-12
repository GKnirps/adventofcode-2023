use std::collections::HashMap;
use std::env;
use std::fs::read_to_string;
use std::iter::once;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;

    let rows = parse(&content)?;

    let arrangements = sum_arrangements(&rows);
    println!("The sum of the possible arrangements of all rows is {arrangements}");

    let unfolded_rows = unfold_rows(&rows);
    let arrangements = sum_arrangements(&unfolded_rows);
    println!("The sum of the possible arrangements of all unfolded rows is {arrangements}");

    Ok(())
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
enum Spring {
    Op,
    Dmg,
    Unk,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct Row {
    spring_conditions: Vec<(Spring, usize)>,
    damaged_groups: Vec<usize>,
}

fn unfold_rows(rows: &[Row]) -> Vec<Row> {
    rows.iter().map(unfold_row).collect()
}

fn unfold_row(row: &Row) -> Row {
    let spring_conditions: Vec<(Spring, usize)> = row
        .spring_conditions
        .iter()
        .copied()
        .chain(once((Spring::Unk, 1)))
        .cycle()
        .take(5 * row.spring_conditions.len() + 4)
        .fold(
            Vec::with_capacity(5 * row.spring_conditions.len() + 4),
            |mut all, cond| {
                if all.is_empty() {
                    all.push(cond);
                } else {
                    let last = all.len() - 1;
                    if cond.0 == all[last].0 {
                        all[last].1 += cond.1;
                    } else {
                        all.push(cond);
                    }
                }
                all
            },
        );
    let damaged_groups = row
        .damaged_groups
        .iter()
        .copied()
        .cycle()
        .take(5 * row.damaged_groups.len())
        .collect();
    Row {
        spring_conditions,
        damaged_groups,
    }
}

fn parse(input: &str) -> Result<Vec<Row>, String> {
    input.lines().map(parse_row).collect()
}

fn parse_row(line: &str) -> Result<Row, String> {
    let (springs, groups) = line
        .split_once(' ')
        .ok_or_else(|| format!("unable to split line '{line}'"))?;

    let (mut spring_conditions, leftover_count, leftover_cond) = springs.chars().try_fold(
        (Vec::with_capacity(groups.len()), 0, Spring::Op),
        |(mut conditions, count, condition), c| {
            let new_condition = match c {
                '.' => Ok(Spring::Op),
                '#' => Ok(Spring::Dmg),
                '?' => Ok(Spring::Unk),
                _ => Err(format!("unknown spring condition '{c}' in '{springs}'")),
            }?;
            if condition == new_condition {
                Ok::<(Vec<(Spring, usize)>, usize, Spring), String>((
                    conditions,
                    count + 1,
                    condition,
                ))
            } else {
                if count != 0 {
                    conditions.push((condition, count));
                }
                Ok((conditions, 1, new_condition))
            }
        },
    )?;

    if leftover_count > 0 {
        spring_conditions.push((leftover_cond, leftover_count));
    }

    let damaged_groups = groups
        .split(',')
        .map(|s| {
            s.parse::<usize>()
                .map_err(|e| format!("unable to parse '{s}' as number in line '{line}': {e}"))
        })
        .collect::<Result<Vec<usize>, String>>()?;
    Ok(Row {
        spring_conditions,
        damaged_groups,
    })
}

fn sum_arrangements(rows: &[Row]) -> u128 {
    rows.iter().map(count_arrangements).sum()
}

type CountCache = HashMap<(Vec<(Spring, usize)>, Vec<usize>), u128>;
fn count_arrangements(row: &Row) -> u128 {
    let mut cache: CountCache = HashMap::with_capacity(4096);
    count_recursive_cached(&mut cache, &row.spring_conditions, &row.damaged_groups)
}

fn count_recursive_cached(
    cache: &mut CountCache,
    springs: &[(Spring, usize)],
    groups: &[usize],
) -> u128 {
    // I could do some stuff to minimize allocation here (or around this), but seriously, I am out
    // of energy
    if let Some(count) = cache.get(&(springs.to_vec(), groups.to_vec())) {
        *count
    } else {
        let count = count_recursive(cache, springs, groups);
        cache.insert((springs.to_vec(), groups.to_vec()), count);
        count
    }
}

fn count_recursive(cache: &mut CountCache, springs: &[(Spring, usize)], groups: &[usize]) -> u128 {
    if let Some(mut group) = groups.first().copied() {
        let mut spring_i = 0;
        while spring_i < springs.len() {
            let len = springs[spring_i].1;
            match springs[spring_i].0 {
                Spring::Op => {}
                Spring::Dmg => {
                    return if len > group {
                        0
                    } else {
                        group -= len;
                        let mut configs = 0;
                        while spring_i < springs.len() {
                            spring_i += 1;
                            if let Some(next_spring) = springs.get(spring_i) {
                                match next_spring.0 {
                                    Spring::Op => {
                                        configs = if group == 0 {
                                            count_recursive_cached(
                                                cache,
                                                &springs[spring_i + 1..],
                                                &groups[1..],
                                            )
                                        } else {
                                            0
                                        };
                                        break;
                                    }
                                    Spring::Dmg => {
                                        if next_spring.1 > group {
                                            configs = 0;
                                            break;
                                        } else {
                                            group -= next_spring.1;
                                        }
                                    }
                                    Spring::Unk => {
                                        if next_spring.1 == group {
                                            group = 0;
                                        } else if next_spring.1 < group {
                                            group -= next_spring.1;
                                        } else if next_spring.1 > group + 1 {
                                            let mut subsprings = springs[spring_i..].to_vec();
                                            subsprings[0].1 -= group + 1;
                                            configs = count_recursive_cached(
                                                cache,
                                                &subsprings,
                                                &groups[1..],
                                            );
                                            break;
                                        } else {
                                            configs = count_recursive_cached(
                                                cache,
                                                &springs[spring_i + 1..],
                                                &groups[1..],
                                            );
                                            break;
                                        }
                                    }
                                }
                            } else {
                                configs = (group == 0 && groups.len() < 2).into();
                            }
                        }
                        configs
                    }
                }
                Spring::Unk => {
                    let mut configs: u128 = 0;
                    for dmg_start in 0..len {
                        for dmg_len in 1..=(len - dmg_start) {
                            if dmg_start + dmg_len == len {
                                if dmg_len == group {
                                    if let Some(next_spring) = springs.get(spring_i + 1) {
                                        if next_spring.0 == Spring::Op {
                                            configs += count_recursive_cached(
                                                cache,
                                                &springs[spring_i + 1..],
                                                &groups[1..],
                                            );
                                        }
                                    } else {
                                        configs += if groups.len() < 2 { 1 } else { 0 };
                                    }
                                } else if dmg_len < group {
                                    if let Some(next_spring) = springs.get(spring_i + 1) {
                                        if next_spring.0 == Spring::Dmg {
                                            let mut subgroups = groups.to_vec();
                                            subgroups[0] -= dmg_len;
                                            configs += count_recursive_cached(
                                                cache,
                                                &springs[spring_i + 1..],
                                                &subgroups,
                                            );
                                        }
                                    }
                                }
                            } else if dmg_len == group {
                                if len == dmg_start + dmg_len + 1 {
                                    configs += count_recursive_cached(
                                        cache,
                                        &springs[spring_i + 1..],
                                        &groups[1..],
                                    );
                                } else {
                                    let mut subsprings = springs[spring_i..].to_vec();
                                    subsprings[0].1 -= dmg_start + dmg_len + 1;
                                    configs +=
                                        count_recursive_cached(cache, &subsprings, &groups[1..]);
                                }
                            }
                        }
                    }
                    // also count options when all unknown springs where counted as operational
                    return configs
                        + count_recursive_cached(cache, &springs[spring_i + 1..], groups);
                }
            }
            spring_i += 1;
        }
        0
    } else {
        springs.iter().all(|spring| spring.0 != Spring::Dmg).into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"???.### 1,1,3
.??..??...?##. 1,1,3
?#?#?#?#?#?#?#? 1,3,1,6
????.#...#... 4,1,1
????.######..#####. 1,6,5
?###???????? 3,2,1
"#;

    #[test]
    fn count_arrangements_works_for_examples() {
        assert_eq!(
            count_arrangements(&parse_row("???.### 1,1,3").expect("expected successful parsing")),
            1
        );
        assert_eq!(
            count_arrangements(
                &parse_row(".??..??...?##. 1,1,3").expect("expected successful parsing")
            ),
            4
        );
        assert_eq!(
            count_arrangements(
                &parse_row("?#?#?#?#?#?#?#? 1,3,1,6").expect("expected successful parsing")
            ),
            1
        );
        assert_eq!(
            count_arrangements(
                &parse_row("????.#...#... 4,1,1").expect("expected successful parsing")
            ),
            1
        );
        assert_eq!(
            count_arrangements(
                &parse_row("????.######..#####. 1,6,5").expect("expected successful parsing")
            ),
            4
        );
        assert_eq!(
            count_arrangements(
                &parse_row("?###???????? 3,2,1").expect("expected successful parsing")
            ),
            10
        );
        assert_eq!(
            count_arrangements(&unfold_row(
                &parse_row("???.### 1,1,3").expect("expected successful parsing")
            )),
            1
        );
        assert_eq!(
            count_arrangements(&unfold_row(
                &parse_row(".??..??...?##. 1,1,3").expect("expected successful parsing")
            )),
            16384
        );
        assert_eq!(
            count_arrangements(&unfold_row(
                &parse_row("?#?#?#?#?#?#?#? 1,3,1,6").expect("expected successful parsing")
            )),
            1
        );
        assert_eq!(
            count_arrangements(&unfold_row(
                &parse_row("????.#...#... 4,1,1").expect("expected successful parsing")
            )),
            16
        );
        assert_eq!(
            count_arrangements(&unfold_row(
                &parse_row("????.######..#####. 1,6,5").expect("expected successful parsing")
            )),
            2500
        );
        assert_eq!(
            count_arrangements(&unfold_row(
                &parse_row("?###???????? 3,2,1").expect("expected successful parsing")
            )),
            506250
        );
        assert_eq!(
            count_arrangements(
                &parse_row("#???.#??#??#????# 4,1,5,1,1").expect("expected successful parsing")
            ),
            3,
        );
        assert_eq!(
            count_arrangements(&parse_row("# 1,1").expect("expected successful parsing")),
            0,
        );
    }

    #[test]
    fn sum_arrangements_works_for_example() {
        // given
        let rows = parse(EXAMPLE).expect("expected successful parsing");

        // when
        let sum = sum_arrangements(&rows);

        // then
        assert_eq!(sum, 21);
    }
}
