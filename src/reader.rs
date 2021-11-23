use crate::err::{ErrType, ParseErr};
use crate::parser::{Action, ActionData, Parser};
use crate::s_time::STime;
use chrono::naive::NaiveDate;

pub struct Group {
    name: String,
    members: Vec<String>,
}

pub struct Clockin {
    time_in: STime,
    time_out: STime,
    job: String,
}

pub struct Reader {
    groups: Vec<Group>,
    clocks: Vec<Clockin>,
}

pub fn read(s: &str) -> Result<Reader, ParseErr> {
    let mut reader = Reader {
        groups: Vec::new(),
        clocks: Vec::new(),
    };
    read_to(s, &mut reader)?;
    Ok(reader)
}

pub fn read_to(s: &str, r: &mut Reader) -> Result<(), ParseErr> {
    let mut p = Parser::new(s);
    let mut year: Option<i32> = None;
    let mut time_in: Option<STime> = None;
    let mut time_out: Option<STime> = None;
    let mut date: Option<NaiveDate> = None;
    let mut job: Option<String> = None;
    let mut tags: Vec<String> = Vec::new();
    loop {
        let action = p.next_action()?;
        match action.ad {
            ActionData::Group(name, members) => r.groups.push(Group { name, members }),
            ActionData::ShortDate(dd, mm) => match &year {
                Some(yr) => date = Some(NaiveDate::from_ymd(*yr, mm, dd)),
                None => return Err(action.as_err(ErrType::YearNotSet)),
            },
            ActionData::LongDate(dd, mm, yy) => date = Some(NaiveDate::from_ymd(yy, mm, dd)),
            ActionData::SetJob(j) => job = Some(j.to_string()),
            ActionData::SetYear(yr) => year = Some(yr),
            ActionData::ClearTags => tags.clear(),
            ActionData::ClearTag(t) => tags.retain(|i| i != t),
            ActionData::Tag(t) => {
                if !tags.iter().find(|i| *i == t).is_some() {
                    tags.push(t.to_string())
                }
            }
            ActionData::Clockin(t) => time_in = Some(t), //TODO make clockin and Out add the item
            ActionData::Clockout(t) => time_out = Some(t),
            ActionData::End => return Ok(()),
        }
    }

    Ok(())
}
