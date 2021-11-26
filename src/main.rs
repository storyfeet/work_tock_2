use clap::{clap_app, crate_version};

pub mod err;
pub mod filter;
pub mod parser;
pub mod reader;
pub mod s_time;
pub mod tokenize;
use chrono::{naive::NaiveDate, Datelike, Weekday};
use clap_conf::*;
use err::CanErr;
use reader::*;
use std::io::Read;

//use std::collections::BTreeMap;

fn main() -> Result<(), err::BoxErr> {
    let clap = clap_app! (
        work_tock =>
        (version : crate_version!())
        (author:"Matthew Stoodley")
        (about:"Clock in and out of work for different jobs/projects")
        (@arg config: -c "Config File")
        (@subcommand complete =>
            (about : "Returns a list of jobs in the current file to aid tab completion")
         )
        (@subcommand in =>
            (@arg job: -j --job +takes_value "The job to clockin to")
            (@arg at: -a --at +takes_value "Time to clockin")
            (@arg date : -d --date +takes_value "Date to clockin")
        )
        (@subcommand out =>
            (@arg long_day:-l --long_day "Allow Days times greater than 24 hours")
            (@arg same_day:-s --same_day "Clock out on same day as last clockin")
            (@arg at:-a --at +takes_value "The time to clockout at")
        )
        (@subcommand write =>
            (@arg format:--format +takes_value "Output format yaml,json,[default] tock")
            (@arg write_file:-f +takes_value "Write output to a file (instead of stdout)")
        )
        (@arg job_filter: -j --job +takes_value #{1,20}"filter by job")
        (@arg group_filter:-g --group +takes_value #{1,20} "filter by group")
        (@arg tag_filter:--tag +takes_value #{1,20} "filter by tag")

        (@arg last: -l "filter by the last day")
        (@arg week_filter: --week +takes_value "filter by week (1-53)")
        (@arg this_week: -w --this_week "filter by this week")
        (@arg month_filter : --month +takes_value "filter by month")
        (@arg this_month:-m --this_month "Filter by this month")
        (@arg day_filter:--dat +takes_value "filter by day")
        (@arg today:-t --today "filter by today")

        (@arg since:--since +takes_value "filter after including date")
        (@arg before:--before +takes_value "filter before not including date")

        (@arg file:-f --file +takes_value "The main file")
        (@arg history:-h --history +takes_value #{0,30} "Other files to process")
        (@arg stdin:--stdin "read stdin instead of any files")
        (@arg print:-p --print "print all selected jobs")
    )
    .get_matches();

    let cfg = clap_conf::with_toml_env(&clap, &["{HOME}/.config/work_tock/init.toml"]);

    if let Some(_) = clap.subcommand_matches("complete") {
        return complete(&cfg);
    }

    let mut clocks = ClockStore::new();

    if clap.is_present("stdin") {
        let mut input = std::io::stdin();
        let mut s = String::new();
        input.read_to_string(&mut s).as_err()?;
        clocks.read(&s)?;
    } else {
        let fname = cfg
            .grab()
            .arg("file")
            .conf("config.file")
            .rep_env()
            .as_err()?;
        let s = load_file(&fname)?;
        clocks.read(&s)?;
    }

    //Build multi filter
    let mut filters: Vec<Box<dyn Fn(&Clock) -> bool>> = Vec::new();

    if let Some(jobs) = clap.values_of("job_filter") {
        filters.push(filter::by_job(jobs))
    }

    if let Some(tags) = clap.values_of("tag_filter") {
        filters.push(filter::by_tag(tags));
    }

    if let Some(grps) = clap.values_of("group_filter") {
        filters.push(filter::by_group(grps, &clocks.groups));
    }

    if let Some(wk) = clap.value_of("week_filter") {
        let start = s_time::week_yr_from_str(wk, Some(s_time::today().year()))?;
        let end = start + chrono::Duration::days(7);
        filters.push(filter::between(start, end));
    }

    if clap.is_present("this_week") {
        let wk = s_time::today().iso_week();
        let mut start = NaiveDate::from_isoywd(wk.year(), wk.week(), Weekday::Mon);
        if clap.is_present("last") {
            start -= chrono::Duration::days(7);
        }
        let end = start + chrono::Duration::days(7);
        filters.push(filter::between(start, end));
    }

    if let Some(mt) = clap.value_of("month_filter") {
        let start = s_time::month_yr_from_str(mt, Some(s_time::today().year()))?;
        let end = s_time::next_month_start(&start);
        filters.push(filter::between(start, end));
    }

    if clap.is_present("this_month") {
        let base = s_time::today().with_day(1).unwrap();
        match clap.is_present("last") {
            true => {
                let start = s_time::prev_month_start(&base);
                filters.push(filter::between(start, base));
            }
            false => {
                let end = s_time::next_month_start(&base);
                filters.push(filter::between(base, end));
            }
        }
    }

    if let Some(df) = clap.value_of("day_filter") {
        let start = s_time::date_from_str(df, Some(s_time::today().year()))?;
        let end = start + chrono::Duration::days(1);
        filters.push(filter::between(start, end));
    }
    if clap.is_present("today") {
        let base = s_time::today();
        match clap.is_present("last") {
            true => filters.push(filter::between(base - chrono::Duration::days(1), base)),
            false => filters.push(filter::between(base, base + chrono::Duration::days(1))),
        }
    }

    if let Some(ds) = clap.value_of("since") {
        let d = s_time::date_from_str(ds, Some(s_time::today().year()))?;
        filters.push(filter::since(d));
    }

    if let Some(ds) = clap.value_of("before") {
        let d = s_time::date_from_str(ds, Some(s_time::today().year()))?;
        filters.push(filter::before(d));
    }

    if filters.len() > 0 {
        clocks.clocks.retain(|c: &Clock| {
            for f in &filters {
                if !f(c) {
                    return false;
                }
            }
            true
        });
    }

    let time_map = clocks.as_time_map(clap.is_present("print"));

    println!("{:?}", time_map);

    Ok(())
}

pub fn complete<'a, H: clap_conf::Getter<'a, String>>(cfg: &'a H) -> Result<(), err::BoxErr> {
    let mut files = history_list(cfg);
    if let Ok(fname) = cfg.grab().arg("file").conf("config.file").rep_env() {
        files.push(fname);
    }
    let mut mp = std::collections::BTreeMap::new();
    for f in files {
        let s = load_file(&f)?;
        let mut p = parser::Parser::new(&s);
        while let Ok(Some(s)) = p.next_ident() {
            match mp.get(s) {
                Some(()) => {}
                None => {
                    mp.insert(s.to_string(), ());
                }
            }
        }
    }

    for k in mp.keys() {
        print!("{} ", k);
    }
    print!("\n");
    Ok(())
}

pub fn history_list<'a, H: clap_conf::Getter<'a, String>>(cfg: &'a H) -> Vec<String> {
    let list = cfg
        .grab_multi()
        .arg("history")
        .conf("config.history")
        .done();
    match list {
        Some(it) => it
            .map(|s| clap_conf::replace::replace_env(&s).unwrap_or(s))
            .collect(),
        None => Vec::new(),
    }
}

fn load_file(fname: &str) -> std::io::Result<String> {
    let mut s = String::new();
    let mut f = std::fs::File::open(fname)?;
    f.read_to_string(&mut s)?;
    Ok(s)
}
