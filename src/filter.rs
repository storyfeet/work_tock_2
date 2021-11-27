use crate::reader::{Clock, ClockStore, Group};
use crate::s_time;
use chrono::{naive::NaiveDate, Datelike, Weekday};
use clap::ArgMatches;

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

pub fn get_args_filter(
    clap: &ArgMatches,
    clocks: &ClockStore,
) -> anyhow::Result<Option<ClockFilter>> {
    //Build multi filter
    let mut filters: Vec<Box<dyn Fn(&Clock) -> bool>> = Vec::new();

    if let Some(jobs) = clap.values_of("job_filter") {
        filters.push(by_job(jobs))
    }

    if let Some(tags) = clap.values_of("tag_filter") {
        filters.push(by_tag(tags));
    }

    if let Some(grps) = clap.values_of("group_filter") {
        filters.push(by_group(grps, &clocks.groups));
    }

    if let Some(wk) = clap.value_of("week_filter") {
        let start = s_time::week_yr_from_str(wk, Some(s_time::today().year()))?;
        let end = start + chrono::Duration::days(7);
        filters.push(between(start, end));
    }

    if clap.is_present("this_week") {
        let wk = s_time::today().iso_week();
        let mut start = NaiveDate::from_isoywd(wk.year(), wk.week(), Weekday::Mon);
        if clap.is_present("last") {
            start -= chrono::Duration::days(7);
        }
        let end = start + chrono::Duration::days(7);
        filters.push(between(start, end));
    }

    if let Some(mt) = clap.value_of("month_filter") {
        let start = s_time::month_yr_from_str(mt, Some(s_time::today().year()))?;
        let end = s_time::next_month_start(&start);
        filters.push(between(start, end));
    }

    if clap.is_present("this_month") {
        let base = s_time::today().with_day(1).unwrap();
        match clap.is_present("last") {
            true => {
                let start = s_time::prev_month_start(&base);
                filters.push(between(start, base));
            }
            false => {
                let end = s_time::next_month_start(&base);
                filters.push(between(base, end));
            }
        }
    }

    if let Some(df) = clap.value_of("day_filter") {
        let start = s_time::date_from_str(df, Some(s_time::today().year()))?;
        let end = start + chrono::Duration::days(1);
        filters.push(between(start, end));
    }
    if clap.is_present("today") {
        let base = s_time::today();
        match clap.is_present("last") {
            true => filters.push(between(base - chrono::Duration::days(1), base)),
            false => filters.push(between(base, base + chrono::Duration::days(1))),
        }
    }

    if let Some(ds) = clap.value_of("since") {
        let d = s_time::date_from_str(ds, Some(s_time::today().year()))?;
        filters.push(since(d));
    }

    if let Some(ds) = clap.value_of("before") {
        let d = s_time::date_from_str(ds, Some(s_time::today().year()))?;
        filters.push(before(d));
    }

    match filters.len() {
        0 => Ok(None),
        _ => Ok(Some(Box::new(move |c: &Clock| {
            for f in &filters {
                if !f(c) {
                    return false;
                }
            }
            true
        }))),
    }
}
