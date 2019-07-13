use super::error::{Error, ErrorKind};
use chrono::prelude::*;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use log::trace;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Deserialize, Serialize)]
pub struct Interval {
    #[serde(default)]
    pub expression: String,

    // 0 to 59
    #[serde(default)]
    pub minutes: Vec<u32>,

    #[serde(default)]
    // 0 to 23
    pub hours: Vec<u32>,

    // 1 to 31
    #[serde(default)]
    pub days: Vec<u32>,

    // 1 to 12
    #[serde(default)]
    pub months: Vec<u32>,

    // 1 (monday) to 7 (sunday)
    #[serde(default)]
    pub weekdays: Vec<u32>,
}

impl Default for Interval {
    fn default() -> Self {
        Interval {
            expression: String::from(""),
            minutes: Vec::new(),
            hours: Vec::new(),
            days: Vec::new(),
            months: Vec::new(),
            weekdays: Vec::new(),
        }
    }
}

impl fmt::Display for Interval {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{:?} {:?} {:?} {:?} {:?}",
            self.minutes, self.hours, self.days, self.months, self.weekdays
        )
    }
}

impl Interval {
    pub fn new(expression: &str) -> Result<Interval, Error> {
        Interval::validate_expression(expression)?;

        let mut iter = expression.split_whitespace().map(|section| {
            let mut numbers = section
                .split(",")
                .filter_map(|item| item.parse().ok())
                .collect::<Vec<u32>>();

            numbers.sort();

            numbers
        });

        let interval = Interval {
            expression: expression.to_string(),
            minutes: iter.next().unwrap(),
            hours: iter.next().unwrap(),
            days: iter.next().unwrap(),
            months: iter.next().unwrap(),
            weekdays: iter.next().unwrap(),
        };

        Interval::validate_interval(&interval)?;

        Ok(interval)
    }

    fn validate_expression(expression: &str) -> Result<(), Error> {
        let regex = r"(\*|(\d,?)+)\s(\*|(\d,?)+)\s(\*|(\d,?)+)\s(\*|(\d,?)+)\s(\*|(\d,?)+)";
        let regex = Regex::new(regex).unwrap();

        if !regex.is_match(expression) {
            return Err(ErrorKind::InvalidIntervalExpression(expression.to_string()))?;
        }

        Ok(())
    }

    fn validate_interval(interval: &Interval) -> Result<(), Error> {
        // 0 to 59
        let minute = interval.minutes.iter().find(|&&m| m > 59);
        // 0 to 23
        let hour = interval.hours.iter().find(|&&h| h > 23);
        // 1 to 31
        let day = interval.days.iter().find(|&&d| d < 1 || d > 31);
        // 1 to 12
        let month = interval.months.iter().find(|&&m| m < 1 || m > 12);
        // 1 (monday) to 7 (sunday)
        let weekday = interval.weekdays.iter().find(|&&w| w < 1 || w > 7);

        if minute.is_some()
            || hour.is_some()
            || day.is_some()
            || month.is_some()
            || weekday.is_some()
        {
            return Err(ErrorKind::InvalidIntervalExpression(
                interval.expression.to_string(),
            ))?;
        }

        Ok(())
    }

    pub fn should_run(&self, previous: DateTime<Utc>, now: DateTime<Utc>) -> bool {
        let next = self.next_time(previous);

        let should = next <= now;

        trace!("Interval\t\t{}", self);
        trace!("Previous\t\t{:?}", previous);
        trace!("Next\t\t{:?}", next);
        trace!("Now\t\t{:?}", now);
        trace!("Should Run\t{}", should);

        should
    }

    pub fn next_time(&self, previous: DateTime<Utc>) -> DateTime<Utc> {
        let next = Utc
            .ymd(previous.year(), previous.month(), previous.day())
            .and_hms(previous.hour(), previous.minute(), 0)
            + Duration::minutes(1);

        let next = self.next_minute_or_carry_hour(next);

        let next = self.next_hour_or_carry_day(next);

        let next = self.next_weekday_or_carry_month(next);

        let next = self.next_day_or_carry_month(next);

        let next = self.next_month_or_carry_year(next);

        next
    }

    fn next_minute_or_carry_hour(&self, date: DateTime<Utc>) -> DateTime<Utc> {
        if self.minutes.is_empty() {
            return date;
        }

        let current = date.minute();
        let &first = self.minutes.first().unwrap();
        let next = self.minutes.iter().find(|&&minute| minute >= current);

        match next {
            Some(&minute) => date.with_minute(minute).unwrap(),
            None => date.with_minute(first).unwrap() + Duration::hours(1),
        }
    }

    fn next_hour_or_carry_day(&self, date: DateTime<Utc>) -> DateTime<Utc> {
        if self.hours.is_empty() {
            return date;
        }

        let current = date.hour();
        let &first = self.hours.first().unwrap();
        let next = self.hours.iter().find(|&&hour| hour >= current);

        match next {
            Some(&hour) => date.with_hour(hour).unwrap(),
            None => date.with_hour(first).unwrap() + Duration::days(1),
        }
    }

    fn next_weekday_or_carry_month(&self, date: DateTime<Utc>) -> DateTime<Utc> {
        if self.weekdays.is_empty() || !self.days.is_empty() {
            return date;
        }

        let current = date.weekday().number_from_monday();
        let &first = self.weekdays.first().unwrap();
        let next = self.weekdays.iter().find(|&&weekday| weekday >= current);

        match next {
            Some(&weekday) => {
                let days = Interval::days_to_weekday(current, weekday);

                date + Duration::days(days)
            }
            None => {
                let days = Interval::days_to_weekday(current, first);

                date + Duration::days(days)
            }
        }
    }

    fn next_day_or_carry_month(&self, date: DateTime<Utc>) -> DateTime<Utc> {
        if self.days.is_empty() || !self.weekdays.is_empty() {
            return date;
        }

        let current = date.day();
        let &first = self.days.first().unwrap();
        let next = self.days.iter().find(|&&day| day >= current);

        match next {
            Some(&day) => {
                let days = Interval::days_to_safe_date(date, date.year(), date.month(), day);

                date + Duration::days(days)
            }
            None => {
                let days = Interval::days_to_safe_date(date, date.year(), date.month() + 1, first);

                date + Duration::days(days)
            }
        }
    }

    fn next_month_or_carry_year(&self, date: DateTime<Utc>) -> DateTime<Utc> {
        if self.months.is_empty() {
            return date;
        }

        let current = date.month();
        let &first = self.months.first().unwrap();
        let next = self.months.iter().find(|&&month| month >= current);

        match next {
            Some(&month) => {
                let days = Interval::days_to_safe_date(date, date.year(), month, date.day());

                date + Duration::days(days)
            }
            None => {
                let days = Interval::days_to_safe_date(date, date.year() + 1, first, date.day());

                date + Duration::days(days)
            }
        }
    }

    fn days_to_weekday(from: u32, to: u32) -> i64 {
        (((to + 7) - from) % 7) as i64
    }

    fn days_to_safe_date(date: DateTime<Utc>, mut year: i32, mut month: u32, day: u32) -> i64 {
        if month > 12 {
            year += 1;
            month -= 12;
        }

        let first_date = date
            .with_day(1)
            .unwrap()
            .with_month(month)
            .unwrap()
            .with_year(year)
            .unwrap();

        let last_day = Interval::last_day_of_month(year, month);
        let last_date = first_date.with_day(last_day).unwrap();

        let safe_date = first_date.with_day(day).unwrap_or(last_date);

        safe_date.signed_duration_since(date).num_days()
    }

    fn last_day_of_month(year: i32, month: u32) -> u32 {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
            .unwrap_or(NaiveDate::from_ymd(year + 1, 1, 1))
            .pred()
            .day()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expression_valid() {
        let interval = Interval::new("0,45,30,15 10,20 * * *").expect("invalid expression");

        assert_eq!(interval.minutes[0], 0);
        assert_eq!(interval.minutes[1], 15);
        assert_eq!(interval.minutes[2], 30);
        assert_eq!(interval.minutes[3], 45);
        assert_eq!(interval.hours[0], 10);
        assert_eq!(interval.hours[1], 20);
        assert_eq!(interval.days.is_empty(), true);
        assert_eq!(interval.months.is_empty(), true);
        assert_eq!(interval.weekdays.is_empty(), true);
    }

    #[test]
    fn expression_invalid_chars() {
        let interval = Interval::new("0,45 a * * *");

        assert_eq!(interval.is_err(), true);
    }

    #[test]
    fn expression_invalid_length() {
        let interval = Interval::new("0,45 * *");

        assert_eq!(interval.is_err(), true);
    }

    #[test]
    fn minute_found() {
        let interval = Interval::new("0,30 * * * *").expect("invalid expression");

        let current_date = Utc.ymd(2019, 7, 1).and_hms(12, 15, 0);
        let next_date = interval.next_minute_or_carry_hour(current_date);

        assert_eq!(next_date, Utc.ymd(2019, 7, 1).and_hms(12, 30, 0));
    }

    #[test]
    fn minute_carry_hour() {
        let interval = Interval::new("0,30 * * * *").expect("invalid expression");

        let current_date = Utc.ymd(2019, 7, 1).and_hms(12, 31, 0);
        let next_date = interval.next_minute_or_carry_hour(current_date);

        assert_eq!(next_date, Utc.ymd(2019, 7, 1).and_hms(13, 0, 0));
    }

    #[test]
    fn minute_carry_hour_and_day() {
        let interval = Interval::new("0,30 * * * *").expect("invalid expression");

        let current_date = Utc.ymd(2019, 7, 1).and_hms(23, 31, 0);
        let next_date = interval.next_minute_or_carry_hour(current_date);

        assert_eq!(next_date, Utc.ymd(2019, 7, 2).and_hms(0, 0, 0));
    }

    #[test]
    fn hour_found() {
        let interval = Interval::new("* 0,12 * * *").expect("invalid expression");

        let current_date = Utc.ymd(2019, 7, 1).and_hms(6, 0, 0);
        let next_date = interval.next_hour_or_carry_day(current_date);

        assert_eq!(next_date, Utc.ymd(2019, 7, 1).and_hms(12, 0, 0));
    }

    #[test]
    fn hour_carry_day() {
        let interval = Interval::new("* 0,12 * * *").expect("invalid expression");

        let current_date = Utc.ymd(2019, 7, 1).and_hms(18, 0, 0);
        let next_date = interval.next_hour_or_carry_day(current_date);

        assert_eq!(next_date, Utc.ymd(2019, 7, 2).and_hms(0, 0, 0));
    }

    #[test]
    fn hour_carry_day_and_month() {
        let interval = Interval::new("* 0,12 * * *").expect("invalid expression");

        let current_date = Utc.ymd(2019, 7, 31).and_hms(18, 0, 0);
        let next_date = interval.next_hour_or_carry_day(current_date);

        assert_eq!(next_date, Utc.ymd(2019, 8, 1).and_hms(0, 0, 0));
    }

    #[test]
    fn day_found() {
        let interval = Interval::new("* * 1,20 * *").expect("invalid expression");

        let current_date = Utc.ymd(2019, 7, 10).and_hms(12, 0, 0);
        let next_date = interval.next_day_or_carry_month(current_date);

        assert_eq!(next_date, Utc.ymd(2019, 7, 20).and_hms(12, 0, 0));
    }

    #[test]
    fn day_carry_month() {
        let interval = Interval::new("* * 1,20 * *").expect("invalid expression");

        let current_date = Utc.ymd(2019, 7, 25).and_hms(12, 0, 0);
        let next_date = interval.next_day_or_carry_month(current_date);

        assert_eq!(next_date, Utc.ymd(2019, 8, 1).and_hms(12, 0, 0));
    }

    #[test]
    fn day_carry_month_and_year() {
        let interval = Interval::new("* * 1,20 * *").expect("invalid expression");

        let current_date = Utc.ymd(2019, 12, 25).and_hms(12, 0, 0);
        let next_date = interval.next_day_or_carry_month(current_date);

        assert_eq!(next_date, Utc.ymd(2020, 1, 1).and_hms(12, 0, 0));
    }

    #[test]
    fn weekday_monday_friday() {
        assert_eq!(Interval::days_to_weekday(1, 5), 4);
    }

    #[test]
    fn weekday_friday_monday() {
        assert_eq!(Interval::days_to_weekday(5, 1), 3);
    }

    #[test]
    fn weekday_sunday_monday() {
        assert_eq!(Interval::days_to_weekday(7, 1), 1);
    }

    #[test]
    fn weekday_sunday_sunday() {
        assert_eq!(Interval::days_to_weekday(7, 7), 0);
    }

    #[test]
    fn weekday_found() {
        let interval = Interval::new("* * * * 1,4").expect("invalid expression");

        let current_date = Utc.ymd(2019, 7, 2).and_hms(12, 0, 0);
        let next_date = interval.next_weekday_or_carry_month(current_date);

        assert_eq!(next_date, Utc.ymd(2019, 7, 4).and_hms(12, 0, 0));
    }

    #[test]
    fn weekday_carry_month() {
        let interval = Interval::new("* * * * 1,4").expect("invalid expression");

        let current_date = Utc.ymd(2019, 7, 30).and_hms(12, 0, 0);
        let next_date = interval.next_weekday_or_carry_month(current_date);

        assert_eq!(next_date, Utc.ymd(2019, 8, 1).and_hms(12, 0, 0));
    }

    #[test]
    fn weekday_carry_month_and_year() {
        let interval = Interval::new("* * * * 1,4").expect("invalid expression");

        let current_date = Utc.ymd(2019, 12, 31).and_hms(12, 0, 0);
        let next_date = interval.next_weekday_or_carry_month(current_date);

        assert_eq!(next_date, Utc.ymd(2020, 1, 2).and_hms(12, 0, 0));
    }

    #[test]
    fn month_found() {
        let interval = Interval::new("* * * 1,6 *").expect("invalid expression");

        let current_date = Utc.ymd(2019, 3, 1).and_hms(12, 0, 0);
        let next_date = interval.next_month_or_carry_year(current_date);

        assert_eq!(next_date, Utc.ymd(2019, 6, 1).and_hms(12, 0, 0));
    }

    #[test]
    fn month_carry_year() {
        let interval = Interval::new("* * * 1,6 *").expect("invalid expression");

        let current_date = Utc.ymd(2019, 8, 1).and_hms(12, 0, 0);
        let next_date = interval.next_month_or_carry_year(current_date);

        assert_eq!(next_date, Utc.ymd(2020, 1, 1).and_hms(12, 0, 0));
    }

    #[test]
    fn next_time_hour() {
        let interval = Interval::new("0 0,6,12,18 * * *").expect("invalid expression");

        let mut next_date = Utc.ymd(2019, 7, 1).and_hms(12, 0, 0);

        next_date = interval.next_time(next_date);
        assert_eq!(next_date, Utc.ymd(2019, 7, 1).and_hms(18, 0, 0));

        next_date = interval.next_time(next_date);
        assert_eq!(next_date, Utc.ymd(2019, 7, 2).and_hms(0, 0, 0));

        next_date = interval.next_time(next_date);
        assert_eq!(next_date, Utc.ymd(2019, 7, 2).and_hms(6, 0, 0));
    }

    #[test]
    fn next_time_weekday() {
        let interval = Interval::new("0 6,18 * * 1").expect("invalid expression");

        let mut next_date = Utc.ymd(2019, 7, 1).and_hms(6, 0, 0);

        next_date = interval.next_time(next_date);
        assert_eq!(next_date, Utc.ymd(2019, 7, 1).and_hms(18, 0, 0));

        next_date = interval.next_time(next_date);
        assert_eq!(next_date, Utc.ymd(2019, 7, 8).and_hms(6, 0, 0));

        next_date = interval.next_time(next_date);
        assert_eq!(next_date, Utc.ymd(2019, 7, 8).and_hms(18, 0, 0));

        next_date = interval.next_time(next_date);
        assert_eq!(next_date, Utc.ymd(2019, 7, 15).and_hms(6, 0, 0));
    }

    #[test]
    fn next_time_day_end_month() {
        let interval = Interval::new("0 0 31 * *").expect("invalid expression");

        let mut next_date = Utc.ymd(2019, 1, 30).and_hms(0, 0, 0);

        next_date = interval.next_time(next_date);
        assert_eq!(next_date, Utc.ymd(2019, 1, 31).and_hms(0, 0, 0));

        next_date = interval.next_time(next_date);
        assert_eq!(next_date, Utc.ymd(2019, 2, 28).and_hms(0, 0, 0));

        next_date = interval.next_time(next_date);
        assert_eq!(next_date, Utc.ymd(2019, 3, 31).and_hms(0, 0, 0));

        next_date = interval.next_time(next_date);
        assert_eq!(next_date, Utc.ymd(2019, 4, 30).and_hms(0, 0, 0));
    }

    #[test]
    fn next_time_month_end_month() {
        let interval = Interval::new("0 0 31 1,2,3,4 *").expect("invalid expression");

        let mut next_date = Utc.ymd(2019, 1, 30).and_hms(0, 0, 0);

        next_date = interval.next_time(next_date);
        assert_eq!(next_date, Utc.ymd(2019, 1, 31).and_hms(0, 0, 0));

        next_date = interval.next_time(next_date);
        assert_eq!(next_date, Utc.ymd(2019, 2, 28).and_hms(0, 0, 0));

        next_date = interval.next_time(next_date);
        assert_eq!(next_date, Utc.ymd(2019, 3, 31).and_hms(0, 0, 0));

        next_date = interval.next_time(next_date);
        assert_eq!(next_date, Utc.ymd(2019, 4, 30).and_hms(0, 0, 0));
    }
}
