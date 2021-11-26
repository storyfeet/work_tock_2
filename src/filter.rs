use crate::reader::{Clock, Group};
use chrono::naive::NaiveDate;

pub type ClockFilter = Box<dyn Fn(&Clock) -> bool>;

pub fn by_job<'a, I: Iterator<Item = &'a str>>(jobs: I) -> ClockFilter {
    let jlist: Vec<String> = jobs.map(str::to_string).collect();
    Box::new(move |c: &Clock| jlist.contains(&c.job))
}

pub fn by_tag<'a, I: Iterator<Item = &'a str>>(tags: I) -> ClockFilter {
    let tlist: Vec<String> = tags.map(str::to_string).collect();
    Box::new(move |c: &Clock| {
        for t in &c.tags {
            if tlist.contains(t) {
                return true;
            }
        }
        false
    })
}

pub fn by_group<'a, I: Iterator<Item = &'a str>>(tags: I, grps: &[Group]) -> ClockFilter {
    let mut v: Vec<String> = Vec::new();
    //TODO dedup the list
    for t in tags {
        for g in grps {
            if g.name == t {
                v.extend(g.members.iter().map(|s| s.clone()));
            }
        }
    }
    Box::new(move |c: &Clock| v.contains(&c.job))
}

pub fn before(d: NaiveDate) -> ClockFilter {
    Box::new(move |c: &Clock| c.date < d)
}

pub fn since(d: NaiveDate) -> ClockFilter {
    Box::new(move |c: &Clock| c.date >= d)
}

pub fn between(f: NaiveDate, t: NaiveDate) -> ClockFilter {
    Box::new(move |c: &Clock| c.date >= f && c.date < t)
}
