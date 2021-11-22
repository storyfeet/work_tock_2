use chrono::naive::NaiveDate;
use chrono::offset::Local;
use chrono::Timelike;
use derive_more::*;
use std::fmt::{Debug, Display, Formatter};
//use std::str::FromStr;

//use crate::err::ParseErr;

#[derive(Copy, Clone, PartialOrd, PartialEq, Eq, Add, Sub, AddAssign, SubAssign)]
pub struct STime(usize); //minutes

impl STime {
    pub fn new(hr: usize, min: usize) -> Self {
        STime(hr * 60 + min)
    }
    pub fn now() -> Self {
        let t = Local::now();
        STime::new(t.time().hour() as usize, t.time().minute() as usize)
    }

    pub fn since(&self, now_date: &NaiveDate, then_time: Self, then_date: &NaiveDate) -> Self {
        let days_between = (*now_date - *then_date).num_days() as usize;
        *self + STime::new(24 * days_between, 0) - then_time
    }
}

/*impl FromStr for STime {
    type Err = TokErr;
    fn from_str(s: &str) -> Result<Self, TokErr> {
        Ok(crate::gob::STIME.parse_s(s).map_err(|e| e.strung())?)
    }
}*/

impl Debug for STime {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:02}:{:02}", self.0 / 60, self.0 % 60)
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
