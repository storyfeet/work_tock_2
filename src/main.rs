use clap::{clap_app, crate_version};

pub mod err;
pub mod filter;
pub mod parser;
pub mod reader;
pub mod s_time;
pub mod tokenize;
use chrono::Datelike;
use clap_conf::*;
use err_tools::*;
use reader::*;
use s_time::STime;
use std::fmt::Write;
use std::io::Read;

use std::str::FromStr;

//use std::collections::BTreeMap;

fn main() -> anyhow::Result<()> {
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

    let (fname, read_state) = if clap.is_present("stdin") {
        let mut input = std::io::stdin();
        let mut s = String::new();
        input.read_to_string(&mut s).e_str("could not read stdin")?;
        (None, clocks.read(&s)?)
    } else {
        let fname = cfg
            .grab()
            .arg("file")
            .conf("config.file")
            .rep_env()
            .e_str("could not get filename")?;
        let s = load_file(&fname)?;
        (Some(fname), clocks.read(&s)?)
    };

    let today = s_time::today();
    if let Some(ci) = &read_state.curr_in {
        if s_time::STime::now() < ci.time_in {
            return e_str("You are clocked in, in the future");
        }
        ci.print();
        clocks.clocks.push(ci.clone().as_clock(STime::now()));
    }

    if let Some(isub) = clap.subcommand_matches("in") {
        let mut ws = "".to_string();
        let indate = match isub.value_of("date") {
            Some(d) => s_time::date_from_str(d, Some(today.year()))?,
            None => today,
        };
        if Some(indate) != read_state.date {
            write!(ws, "{}\n", indate.format("%d/%m/%Y"))?;
        }
        ws.push('\t');
        let job = isub
            .value_of("job")
            .map(String::from)
            .or(read_state.job.clone())
            .e_str("No Job provided for clock in")?;
        if Some(&job) != read_state.job.as_ref() {
            write!(ws, "{},", job)?;
        }
        let time = match isub.value_of("at") {
            Some(t) => s_time::STime::from_str(t)?,
            None => match isub.value_of("date") {
                Some(_) => e_str("Date Time required when date given")?,
                None => s_time::STime::now(),
            },
        };
        write!(ws, "{}", time)?;

        match &fname {
            Some(nm) => {
                let mut f = std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .append(true)
                    .open(nm)?;
                std::io::Write::write_fmt(&mut f, format_args!("{}\n", ws))?;
            }
            None => {
                println!("{}", ws);
            }
        }
    }

    if let Some(f) = filter::get_args_filter(&clap, &clocks)? {
        clocks.clocks.retain(f);
    }

    let time_map = clocks.as_time_map(clap.is_present("print"));

    println!("{:?}", time_map);

    Ok(())
}

pub fn complete<'a, H: clap_conf::Getter<'a, String>>(cfg: &'a H) -> anyhow::Result<()> {
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
