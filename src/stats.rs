use std::fmt;
use std::ops::Div;

pub struct Stats {
    avg_time: u128,
    median_time: u128,
    min_time: u128,
    max_time: u128,
    success_count: usize,
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        writeln!(f, "Successfully completed {} requests", self.success_count)?;
        writeln!(f, "Avg response time: {}ms", self.avg_time)?;
        writeln!(f, "Median response time: {}ms", self.median_time)?;
        writeln!(f, "Min response time: {}ms", self.min_time)?;
        writeln!(f, "Max response time: {}ms", self.max_time)
    }
}

fn median<T>(mut times: Vec<T>) -> T
where
    T: Div + Ord + Copy,
{
    times.sort();
    let mid_idx = times.len() / 2;
    times[mid_idx]
}

pub fn calc_stats(results: Vec<Result<u128, String>>) -> Stats {
    let mut sum = 0;
    let mut idx: usize = 0;
    let mut max_time = 0;
    let mut min_time = u128::max_value();
    let mut times = Vec::new();
    for result in results {
        match result {
            Ok(time) => {
                idx += 1;
                sum += time;
                if time < min_time {
                    min_time = time;
                }
                if time > max_time {
                    max_time = time;
                }
                times.push(time);
            }
            Err(e) => {
                println!(
                    "Not adding request stats due to error during request: {}",
                    e
                );
            }
        }
    }
    let avg = sum / idx as u128;
    let median_time = median(times);
    Stats {
        avg_time: avg,
        median_time,
        min_time,
        max_time,
        success_count: idx,
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn median_sort() {
        let v = vec![2, 1, 4, 3];
        assert_eq!(median(v), 3);
    }

    #[test]
    fn median_odd() {
        let v = vec![0, 4, 123, 5, 8];
        assert_eq!(median(v), 5);
    }

    #[test]
    fn calc() {
        let results = vec![
            Result::Ok(3),
            Result::Err("Error".to_owned()),
            Result::Ok(5),
            Result::Ok(7),
        ];
        let stats = calc_stats(results);
        assert_eq!(stats.avg_time, 5);
        assert_eq!(stats.max_time, 7);
        assert_eq!(stats.min_time, 3);
        assert_eq!(stats.median_time, 5);
        assert_eq!(stats.success_count, 3);
    }
}
