use crate::err::{ClockErr, ClockErrType, ErrType, ParseErr};
use crate::parser::{ActionData, Parser};
use crate::s_time::{today, STime};
use chrono::naive::NaiveDate;
use std::collections::BTreeMap;
//use std::fmt::{self, Display};

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
#[derive(Clone)]
pub struct Clockin {
    pub time_in: STime,
    pub date: NaiveDate,
    pub job: String,
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
    pub fn print(&self) {
        let now = STime::now();
        let today = today();
        let d_string = match self.date {
            d if d == today => "today".to_string(),
            d if d + chrono::Duration::days(1) == today => "yesterday".to_string(),
            d => {
                format!("{}", d.format("&d/%m/%Y"))
            }
        };
        println!(
            "You have been clocked in for {}, since {}: {} for {} Hours",
            self.job,
            d_string,
            self.time_in,
            now.since(&today, self.time_in, &self.date),
        );
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

    pub fn as_time_map(self, print: bool) -> Result<BTreeMap<String, STime>, ClockErr> {
        let mut mp = BTreeMap::new();
        let mut tot_time = STime::new(0, 0);
        let mut last_date = NaiveDate::from_ymd(1, 1, 1);
        for c in self.clocks {
            if c.date != last_date {
                last_date = c.date;
                if print {
                    println!("{}", last_date.format("%d/%m/%Y"));
                }
            }
            if c.time_in > c.time_out {
                return Err(ClockErr {
                    clock: c,
                    etype: ClockErrType::OutBeforeIn,
                });
            }
            let inc = c.time_out - c.time_in;
            tot_time += inc;
            match mp.get_mut(&c.job) {
                Some(tot) => {
                    *tot += inc;
                    if print {
                        println!(
                            "  {:<15}: {}-{} = {} => {}   {}",
                            c.job, c.time_in, c.time_out, inc, *tot, tot_time
                        );
                    }
                }
                None => {
                    if print {
                        println!(
                            "  {:<15}: {}-{} = {} => {}   {}",
                            c.job, c.time_in, c.time_out, inc, inc, tot_time
                        );
                    }
                    mp.insert(c.job.to_string(), c.time_out - c.time_in);
                }
            }
        }
        Ok(mp)
    }
}
