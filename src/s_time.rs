use crate::err::{self, CanErr};
use chrono::naive::NaiveDate;
use chrono::offset::Local;
use chrono::Timelike;
use derive_more::*;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

//use crate::err::ParseErr;

#[derive(Copy, Clone, PartialOrd, PartialEq, Eq, Add, Sub, AddAssign, SubAssign)]
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
}

impl FromStr for STime {
    type Err = err::BoxErr;
    fn from_str(s: &str) -> Result<Self, err::BoxErr> {
        let mut ss = s.split(":");
        let hr: u32 = num_from_split(&mut ss)?;
        let min: u32 = num_from_split(&mut ss)?;
        if min >= 60 {
            return Err(err::ErrType::MinutesOver60).as_err();
        }
        Ok(STime(hr * 60 + min))
    }
}

pub fn date_from_str(s: &str, def_year: Option<i32>) -> Result<NaiveDate, err::BoxErr> {
    let mut ss = s.split("/");
    let dd: u32 = num_from_split(&mut ss)?;
    let mm: u32 = num_from_split(&mut ss)?;
    let yy = num_from_split(&mut ss);
    match yy {
        Ok(y) => Ok(NaiveDate::from_ymd(y, mm, dd)),
        Err(e) => def_year.map(|y| NaiveDate::from_ymd(y, mm, dd)).ok_or(e),
    }
}

fn num_from_split<'a, I: Iterator<Item = &'a str>, N: FromStr>(i: &mut I) -> Result<N, err::BoxErr>
where
    <N as FromStr>::Err: std::error::Error + 'static,
{
    i.next()
        .ok_or(err::ErrType::MissingItem)
        .as_err()?
        .parse()
        .as_err()
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
