use anyhow::{anyhow, Result};
use std::collections::HashSet;

/// Parse a number range expression like "1,3-5,7" into a sorted vector of numbers
pub fn parse_number_range(range_str: &str) -> Result<Vec<usize>> {
    let mut numbers = HashSet::new();

    for part in range_str.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        if part.contains('-') {
            let range_parts: Vec<&str> = part.split('-').collect();
            if range_parts.len() != 2 {
                return Err(anyhow!("Invalid range format: {}", part));
            }

            let start = range_parts[0]
                .trim()
                .parse::<usize>()
                .map_err(|_| anyhow!("Invalid number in range: {}", range_parts[0]))?;

            let end = range_parts[1]
                .trim()
                .parse::<usize>()
                .map_err(|_| anyhow!("Invalid number in range: {}", range_parts[1]))?;

            if start > end {
                return Err(anyhow!("Invalid range: {} > {}", start, end));
            }

            for i in start..=end {
                numbers.insert(i);
            }
        } else {
            let num = part
                .parse::<usize>()
                .map_err(|_| anyhow!("Invalid number: {}", part))?;
            numbers.insert(num);
        }
    }

    let mut result: Vec<usize> = numbers.into_iter().collect();
    result.sort();

    if result.is_empty() {
        return Err(anyhow!("No valid numbers in range expression"));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_number() {
        assert_eq!(parse_number_range("5").unwrap(), vec![5]);
    }

    #[test]
    fn test_comma_separated() {
        assert_eq!(parse_number_range("1,3,5").unwrap(), vec![1, 3, 5]);
    }

    #[test]
    fn test_range() {
        assert_eq!(parse_number_range("1-5").unwrap(), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_combined() {
        assert_eq!(parse_number_range("1,3-5,7").unwrap(), vec![1, 3, 4, 5, 7]);
    }

    #[test]
    fn test_duplicates() {
        assert_eq!(parse_number_range("1,1,3-5,3").unwrap(), vec![1, 3, 4, 5]);
    }

    #[test]
    fn test_invalid_format() {
        assert!(parse_number_range("1-5-7").is_err());
    }

    #[test]
    fn test_invalid_range() {
        assert!(parse_number_range("5-1").is_err());
    }

    #[test]
    fn test_invalid_number() {
        assert!(parse_number_range("abc").is_err());
    }
}
