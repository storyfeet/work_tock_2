use crate::err::{ClockErr, ClockErrType, ErrType, ParseErr};
use crate::parser::{ActionData, Parser};
use crate::s_time::STime;
use chrono::naive::NaiveDate;
use std::collections::BTreeMap;

pub struct Group {
    pub name: String,
    pub members: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Clock {
    pub time_in: STime,
    pub time_out: STime,
    pub date: NaiveDate,
    pub job: String,
    pub tags: Vec<String>,
}

//Half a clock
pub struct Clockin {
    time_in: STime,
    date: NaiveDate,
    job: String,
    tags: Vec<String>,
}

impl Clockin {
    pub fn as_clock(self, time_out: STime) -> Clock {
        Clock {
            time_in: self.time_in,
            time_out,
            date: self.date,
            job: self.job,
            tags: self.tags,
        }
    }
}

pub struct ClockStore {
    pub groups: Vec<Group>,
    pub clocks: Vec<Clock>,
}

pub struct ReadState {
    pub year: Option<i32>,
    pub date: Option<NaiveDate>,
    pub job: Option<String>,
    pub tags: Vec<String>,
    pub curr_in: Option<Clockin>,
}

impl ReadState {
    pub fn new() -> Self {
        ReadState {
            year: None,
            date: None,
            job: None,
            tags: Vec::new(),
            curr_in: None,
        }
    }
}

impl ClockStore {
    pub fn new() -> Self {
        ClockStore {
            groups: Vec::new(),
            clocks: Vec::new(),
        }
    }

    pub fn read(&mut self, s: &str) -> Result<ReadState, ParseErr> {
        let mut p = Parser::new(s);
        let mut rs = ReadState::new();

        loop {
            let action = p.next_action()?;
            match action.ad {
                ActionData::Group(name, members) => self.groups.push(Group { name, members }),
                ActionData::ShortDate(dd, mm) => match &rs.year {
                    Some(yr) => rs.date = Some(NaiveDate::from_ymd(*yr, mm, dd)),
                    None => return Err(action.as_err(ErrType::YearNotSet)),
                },
                ActionData::LongDate(dd, mm, yy) => rs.date = Some(NaiveDate::from_ymd(yy, mm, dd)),
                ActionData::SetJob(j) => rs.job = Some(j.to_string()),
                ActionData::SetYear(yr) => rs.year = Some(yr),
                ActionData::ClearTags => rs.tags.clear(),
                ActionData::ClearTag(t) => rs.tags.retain(|i| i != t),
                ActionData::Tag(t) => {
                    if !rs.tags.iter().find(|i| *i == t).is_some() {
                        rs.tags.push(t.to_string())
                    }
                }
                ActionData::Clockin(t) => {
                    if let Some(last) = rs.curr_in.take() {
                        //TODO add checks
                        self.clocks.push(last.as_clock(t));
                    }
                    rs.curr_in = Some(Clockin {
                        time_in: t,
                        date: rs
                            .date
                            .clone()
                            .take()
                            .ok_or(action.as_err(ErrType::DateNotSet))?,
                        job: rs
                            .job
                            .clone()
                            .take()
                            .ok_or(action.as_err(ErrType::JobNotSet))?,
                        tags: rs.tags.clone(),
                    })
                }
                ActionData::Clockout(t) => {
                    match rs.curr_in.take() {
                        //with checks
                        Some(i) => self.clocks.push(i.as_clock(t)),
                        None => return Err(action.as_err(ErrType::ClockinNotSet)),
                    }
                }
                ActionData::End => return Ok(rs),
            }
        }
    }

    pub fn as_time_map(self) -> Result<BTreeMap<String, STime>, ClockErr> {
        let mut mp = BTreeMap::new();
        for c in self.clocks {
            if c.time_in > c.time_out {
                return Err(ClockErr {
                    clock: c,
                    etype: ClockErrType::OutBeforeIn,
                });
            }
            match mp.get_mut(&c.job) {
                Some(tot) => {
                    *tot += c.time_out - c.time_in;
                }
                None => {
                    mp.insert(c.job.to_string(), c.time_out - c.time_in);
                }
            }
        }
        Ok(mp)
    }
}
