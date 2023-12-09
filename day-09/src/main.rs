use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let values = parse_values(&content)?;

    if let Some((pred_prev, pred_next)) = prediction_sum(&values) {
        println!("The sum of predicted next values is {pred_next}");
        println!("The sum of predicted previous values is {pred_prev}");
    } else {
        println!("It's unpredicatable…");
    }

    Ok(())
}

fn parse_line(line: &str) -> Result<Vec<i64>, String> {
    line.split_whitespace()
        .map(|s| {
            s.parse::<i64>()
                .map_err(|e| format!("unable to parse value '{s}': {e}"))
        })
        .collect()
}

fn parse_values(input: &str) -> Result<Vec<Vec<i64>>, String> {
    input.lines().map(parse_line).collect()
}

fn predict(values: &[i64]) -> Option<(i64, i64)> {
    // let's do a primitive approach first
    let mut differences: Vec<Vec<i64>> = Vec::with_capacity(values.len());
    let mut diff_values: &[i64] = values;
    while diff_values.iter().any(|v| *v != 0) {
        differences.push(diff_values.windows(2).map(|v| v[1] - v[0]).collect());
        diff_values = &differences[differences.len() - 1];
    }

    let mut first_diff: i64 = 0;
    let mut last_diff: i64 = 0;
    for diffs in differences.iter().rev() {
        first_diff = diffs.first()? - first_diff;
        last_diff += diffs.last()?;
    }
    let pred_prev = values.first().map(|v| v - first_diff)?;
    let pred_next = values.last().map(|v| v + last_diff)?;

    Some((pred_prev, pred_next))
}

fn prediction_sum(values: &[Vec<i64>]) -> Option<(i64, i64)> {
    values
        .iter()
        .map(|v| predict(v))
        .try_fold((0, 0), |(sum_p, sum_n), next| {
            let (p, n) = next?;
            Some((sum_p + p, sum_n + n))
        })
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"0 3 6 9 12 15
1 3 6 10 15 21
10 13 16 21 30 45
"#;

    #[test]
    fn predict_works_for_example() {
        assert_eq!(predict(&[0, 3, 6, 9, 12, 15]), Some((-3, 18)));
        assert_eq!(predict(&[1, 3, 6, 10, 15, 21]), Some((0, 28)));
        assert_eq!(predict(&[10, 13, 16, 21, 30, 45]), Some((5, 68)));
    }

    #[test]
    fn prediction_sum_works_for_example() {
        // given
        let values = parse_values(EXAMPLE).expect("expected successful parsing");

        // when
        let sum = prediction_sum(&values);

        // then
        assert_eq!(sum, Some((2, 114)));
    }
}
