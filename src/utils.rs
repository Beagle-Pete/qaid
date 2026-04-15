use core::str;
use std::collections::HashMap;
use std::hash::Hash;

use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use dateparser::parse_with;

pub fn get_duplicates<T: Eq + PartialEq + Hash + Clone>(data: &[T]) -> Vec<T>{
    let mut map: HashMap<T, bool> = HashMap::with_capacity(data.len());

    for el in data.iter().cloned() {
        map.entry(el)
            .and_modify(|e| *e = true)
            .or_insert(false);
    }

    map.into_iter()
        .filter(|el| el.1)
        .map(|el| el.0)
        .collect()
}

pub fn parse_date(input : &str) -> Option<NaiveDateTime> {
    let input= input.trim();
    let midnight = NaiveTime::from_hms_opt(0, 0, 0).unwrap_or_default();

    // 1. Try to interpret as a serial number (e.g. "46024" or "46024.05")
    if let Ok(serial) = input.parse::<f64>() {
        return serial_to_datetime(serial);
    }
    
    // Try dateparser (handles RFC 3339, RFC 2822, named timezones, etc.)
    // parse_with pins the default time to midnight when no time is present.
    if let Ok(dt) = parse_with(input, &Utc, midnight) {
        return Some(dt.naive_utc());
    }

    // 3. Fallback: try chrono's NaiveDateTime for timezone-naive ISO 8601
    // e.g. "2023-06-15T14:30:00" or "2023-06-15T14:30:00.123"
    // dateparser rejects these because there's no timezone indicator.
    let formats_to_try = [
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M:%S%.f",  // with fractional seconds
        "%Y-%m-%d %H:%M:%S",     // space separator variant
        "%Y-%m-%d %H:%M:%S%.f",
    ];

    for fmt in formats_to_try {
        if let Ok(dt) = NaiveDateTime::parse_from_str(input, fmt) {
            return Some(dt);
        }
    }

    None
}

fn serial_to_datetime(serial: f64) -> Option<NaiveDateTime> {
    let days = serial.floor() as i64;
    let adjusted_days = if days >= 60 { days - 2 } else { days - 1 };

    let epoch = NaiveDate::from_ymd_opt(1900, 1, 1)?;
    let date = epoch.checked_add_signed(Duration::days(adjusted_days))?;

    let fraction = serial.fract();
    let total_seconds = (fraction * 86_400.0).round() as u32;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    let time = NaiveTime::from_hms_opt(hours, minutes, seconds)?;
    Some(date.and_time(time))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_duplicates_test() {
        let vec = vec![1, 2, 3];
        assert_eq!(get_duplicates(&vec), []);

        let vec = vec![1, 2, 3, 3];
        assert_eq!(get_duplicates(&vec), [3]);

        let vec = vec![1, 2, 3, 3, 3, 4, 4];
        assert!(get_duplicates(&vec).contains(&3));
        assert!(get_duplicates(&vec).contains(&4));

        let vec = vec!["one", "two", "three", "three", "three", "four"];
        assert_eq!(get_duplicates(&vec), ["three"]);
    }

    #[test]
    fn date_parse_test() {
        assert_eq!(
            parse_date("12/05/25").unwrap(),
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2025, 12, 5).unwrap(), 
                NaiveTime::from_hms_micro_opt(0, 0, 0, 0).unwrap()
        ));

        assert_eq!(
            parse_date("June 15, 2023").unwrap(),
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2023, 6, 15).unwrap(), 
                NaiveTime::from_hms_micro_opt(0, 0, 0, 0).unwrap()
        ));

        assert_eq!(
            parse_date("June 15, 2023 3:45pm").unwrap(),
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2023, 6, 15).unwrap(), 
                NaiveTime::from_hms_micro_opt(15, 45, 0, 0).unwrap()
        ));

        assert_eq!(
            parse_date("2023-06-15T14:30:00Z").unwrap(),
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2023, 6, 15).unwrap(), 
                NaiveTime::from_hms_micro_opt(14, 30, 0, 0).unwrap()
        ));

        assert_eq!(
            parse_date("2023-06-15T14:30:00").unwrap(),
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2023, 6, 15).unwrap(), 
                NaiveTime::from_hms_micro_opt(14, 30, 0, 0).unwrap()
        ));

        assert_eq!(
            parse_date("May 26, 2021, 12:49 AM PDT").unwrap(),
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2021, 5, 26).unwrap(), 
                NaiveTime::from_hms_micro_opt(7, 49, 0, 0).unwrap()
        ));

        assert_eq!(
            parse_date("44680.7042939815").unwrap(),
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2022, 4, 29).unwrap(), 
                NaiveTime::from_hms_micro_opt(16, 54, 11, 0).unwrap()
        ));

        
    }
}