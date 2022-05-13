use crate::err::{self, ErrType};
use chrono::naive::NaiveDate;
use chrono::offset::Local;
use chrono::{Datelike, Timelike, Weekday};
use derive_more::*;
use std::cmp::{Ordering, PartialOrd};
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Moment {
    pub t: STime,
    pub d: NaiveDate,
}

impl Moment {
    pub fn now() -> Self {
        let now = Local::now();
        Moment {
            t: STime::new(now.time().hour() as u32, now.time().minute() as u32),
            d: now.date().naive_local(),
        }
    }
    pub fn new(d: NaiveDate, t: STime) -> Self {
        Moment { d, t }
    }
    pub fn print_relative(&self, now: &Moment) -> String {
        match self.d {
            d if d == now.d => format!("today : {}", self.t),
            d if d + chrono::Duration::days(1) == now.d => format!("yesterday : {}", self.t),
            d => format!("{} : {}", d.format("&d/%m/%Y"), self.t),
        }
    }

    pub fn print(&self) -> String {
        self.print_relative(&Self::now())
    }

    pub fn time_since(&self, prev: &Moment) -> STime {
        let days_between = (self.d - prev.d).num_days() as u32;
        (STime::new(24 * days_between, 0) + self.t) - prev.t
    }
}

impl PartialOrd for Moment {
    fn partial_cmp(&self, b: &Self) -> Option<Ordering> {
        Some(self.cmp(b))
    }
}

impl Ord for Moment {
    fn cmp(&self, b: &Self) -> Ordering {
        match self.d.cmp(&b.d) {
            Ordering::Equal => self.t.cmp(&b.t),
            n => return n,
        }
    }
}

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Add, Sub, AddAssign, SubAssign)]
pub struct STime(u32); //minutes

impl STime {
    pub fn new(hr: u32, min: u32) -> Self {
        STime(hr * 60 + min)
    }
    pub fn now() -> Self {
        let t = Local::now();
        STime::new(t.time().hour() as u32, t.time().minute() as u32)
    }

    pub fn since(&self, now_date: &NaiveDate, then_time: Self, then_date: &NaiveDate) -> Self {
        let days_between = (*now_date - *then_date).num_days() as u32;
        *self + STime::new(24 * days_between, 0) - then_time
    }

    pub fn earlier(&self, b: Self) -> Self {
        if b.0 > self.0 {
            return Self::new(0, 0);
        }
        Self(self.0 - b.0)
    }
}

impl FromStr for STime {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> anyhow::Result<Self> {
        let mut ss = s.split(":");
        let hr: u32 = num_from_split(&mut ss)?;
        let min: u32 = num_from_split(&mut ss)?;
        if min >= 60 {
            return Err(err::ErrType::MinutesOver60.into());
        }
        Ok(STime(hr * 60 + min))
    }
}

pub fn date_from_str(s: &str, def_year: Option<i32>) -> anyhow::Result<NaiveDate> {
    let mut ss = s.split("/");
    let dd: u32 = num_from_split(&mut ss)?;
    let mm: u32 = num_from_split(&mut ss)?;
    match num_from_split(&mut ss) {
        Ok(y) => NaiveDate::from_ymd_opt(y, mm, dd)
            .ok_or(ErrType::DateNotValid)
            .map_err(|e| e.into()),
        Err(e) => match def_year {
            Some(y) => NaiveDate::from_ymd_opt(y, mm, dd)
                .ok_or(ErrType::DateNotValid)
                .map_err(|e| e.into()),
            None => Err(e),
        },
    }
}

pub fn week_yr_from_str(s: &str, def_year: Option<i32>) -> anyhow::Result<NaiveDate> {
    let mut ss = s.split("/");
    let dd: u32 = num_from_split(&mut ss)?;
    match num_from_split(&mut ss) {
        Ok(y) => NaiveDate::from_isoywd_opt(y, dd, Weekday::Mon)
            .ok_or(ErrType::DateNotValid)
            .map_err(|e| e.into()),
        Err(e) => match def_year {
            Some(y) => NaiveDate::from_isoywd_opt(y, dd, Weekday::Mon)
                .ok_or(ErrType::DateNotValid)
                .map_err(|e| e.into()),
            None => Err(e),
        },
    }
}

pub fn prev_month_start(dt: &NaiveDate) -> NaiveDate {
    let mut m = dt.month();
    if m == 0 {
        m = 12
    }
    NaiveDate::from_ymd(dt.year() - ((m / 12) as i32), m, 1)
}

pub fn next_month_start(dt: &NaiveDate) -> NaiveDate {
    let m = dt.month();
    NaiveDate::from_ymd(dt.year() + ((m / 12) as i32), (m % 12) + 1, 1)
}

pub fn month_yr_from_str(s: &str, def_year: Option<i32>) -> anyhow::Result<NaiveDate> {
    let mut ss = s.split("/");
    let mm: u32 = num_from_split(&mut ss)?;
    match num_from_split(&mut ss) {
        Ok(y) => NaiveDate::from_ymd_opt(y, mm, 1)
            .ok_or(ErrType::DateNotValid)
            .map_err(|e| e.into()),
        Err(e) => match def_year {
            Some(y) => NaiveDate::from_ymd_opt(y, mm, 1)
                .ok_or(ErrType::DateNotValid)
                .map_err(|e| e.into()),
            None => Err(e),
        },
    }
}

pub fn today() -> NaiveDate {
    chrono::offset::Local::today().naive_local()
}

fn num_from_split<'a, I: Iterator<Item = &'a str>, N: FromStr>(i: &mut I) -> anyhow::Result<N>
where
    <N as FromStr>::Err: std::error::Error + 'static + Sync + Send,
{
    match i.next() {
        Some(v) => v.parse().map_err(|e: <N as FromStr>::Err| e.into()),
        None => Err(err::ErrType::MissingItem.into()),
    }
}

impl Debug for STime {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:02}:{:02}", self.0 / 60, (self.0 % 60))
    }
}

impl Display for STime {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    pub fn test_stime_parse() {
        assert!("243430343090349309309430334390:54"
            .parse::<STime>()
            .is_err());
        assert_eq!("24:54".parse(), Ok(STime::new(24, 54)));
    }
}
