use clap::{clap_app, crate_version};

pub mod err;
pub mod filter;
pub mod moment;
pub mod parser;
pub mod reader;
pub mod tokenize;
use chrono::Datelike;
use clap_conf::*;
use err_tools::*;
use moment::{Moment, STime};
use reader::*;
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
            (about:"Clock into a job (automatically clocks out of the current job)")
            (@arg job: -j --job +takes_value "The job to clockin to")
            (@arg at: -a --at +takes_value "Time to clockin")
            (@arg date : -d --date +takes_value "Date to clockin")
        )
        (@subcommand out =>
            (about:"Clock out of the current job")
            (@arg long_day:-l --long_day "Allow Days times greater than 24 hours")
            (@arg same_day:-s --same_day "Clock out on same day as last clockin")
            (@arg date : -d --date "Set the date of the clock out")
            (@arg at:-a --at +takes_value "The time to clockout at")
        )
        (@subcommand last =>
            (about:"Clock in a duration ago and out again")
            (@arg duration:+required "The duration")
            (@arg job:-j --job +takes_value "The job to clock in and out of")
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
        if let Some(v) = cfg.grab_multi().arg("history").conf("history").done() {
            for f in v {
                let s = load_file(&f)?;
                let rs = clocks
                    .read(&s)
                    .e_string(format!("Error in history file : {}", f))?;
                if let Some(_in) = rs.curr_in {
                    return e_string(format!("History file clocked in at end : {}", f));
                }
            }
        }
        let fname = cfg
            .grab()
            .arg("file")
            .conf("config.file")
            .rep_env()
            .e_str("could not get filename")?;
        let s = load_file(&fname)?;
        (Some(fname), clocks.read(&s)?)
    };

    //let today = s_time::today();
    if let Some(ci) = &read_state.curr_in {
        if STime::now() < ci.c_in.t {
            return e_str("You are clocked in, in the future");
        }
        ci.print();
        clocks.clocks.push(ci.clone().as_clock(STime::now()));
    }

    if let Some(isub) = clap.subcommand_matches("in") {
        clock_in(isub, read_state, &fname)?;
        return Ok(());
    }

    if let Some(osub) = clap.subcommand_matches("out") {
        clock_out(osub, &read_state, &fname)?;
        return Ok(());
    }

    if let Some(lsub) = clap.subcommand_matches("last") {
        clock_last(lsub, &read_state, &fname)?;
        return Ok(());
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

pub fn clock_in(
    isub: &clap::ArgMatches,
    read_state: reader::ReadState,
    fname: &Option<String>,
) -> anyhow::Result<()> {
    let today = moment::today();
    let mut ws = "".to_string();
    let indate = match isub.value_of("date") {
        Some(d) => moment::date_from_str(d, Some(today.year()))?,
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
        Some(t) => STime::from_str(t)?,
        None => match isub.value_of("date") {
            Some(_) => e_str("Date Time required when date given")?,
            None => STime::now(),
        },
    };
    write!(ws, "{}", time)?;
    print_result(fname, &ws)
}

pub fn clock_out(
    osub: &clap::ArgMatches,
    rs: &ReadState,
    fname: &Option<String>,
) -> anyhow::Result<()> {
    let curr_in = match &rs.curr_in {
        Some(i) => i,
        None => return err_tools::e_str("Cannot Clock out if not clocked in"),
    };

    let now = Moment::now();
    let dfs = match osub.value_of("date") {
        Some(d) => Some(moment::date_from_str(d, Some(now.d.year()))?),
        None => None,
    };

    let out = match (dfs, osub.value_of("at"), osub.is_present("same_day")) {
        (_, Some(ts), true) => Moment::new(curr_in.c_in.d, STime::from_str(ts)?),
        (Some(ds), Some(ts), false) => Moment::new(ds, STime::from_str(ts)?),
        (Some(_), None, _) | (_, None, true) => {
            return e_str("Cannot use current time when applying a previous date")
        }
        (None, Some(ts), false) => Moment::new(now.d, STime::from_str(ts)?),
        (None, None, false) => now,
    };
    let otime = match (
        (out.d - curr_in.c_in.d).num_days(),
        osub.is_present("long_day"),
    ) {
        (n, _) if n < 0 => return e_str("Cannot Clock out before clock in"),
        (0, _) if out.t < curr_in.c_in.t => return e_str("Cannot Clock out before Clock in"),
        (0, _) => out.t,
        (n, true) => out.t + STime::new(n as u32 * 24, 0),
        (_, false) => return e_str("If clocking out the next day, please mark -l for long_day"),
    };

    match &fname {
        Some(nm) => {
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .append(true)
                .open(nm)?;
            std::io::Write::write_fmt(&mut f, format_args!("  -{}\n", otime))?;
        }
        None => {
            println!("  -{}", otime);
        }
    }
    Ok(())
}

pub fn clock_last(
    osub: &clap::ArgMatches,
    rs: &ReadState,
    fname: &Option<String>,
) -> anyhow::Result<()> {
    if let Some(i) = &rs.curr_in {
        return e_string(format!("Currently clocked in for {:?}", i));
    }
    let today = moment::today();
    let mut ws = "".to_string();
    if Some(today) != rs.date {
        write!(ws, "{}\n", today.format("%d/%m/%Y"))?;
    }
    write!(ws, "\t")?;
    let job = osub
        .value_of("job")
        .map(String::from)
        .or(rs.job.clone())
        .e_str("No Job provided for clock in")?;
    if Some(&job) != rs.job.as_ref() {
        write!(ws, "{},", job)?;
    }

    let duration = match osub.value_of("duration") {
        Some("hour") => STime::new(1, 0),
        Some("half") => STime::new(0, 30),
        Some(s) => STime::from_str(s)?,
        None => STime::new(1, 0),
    };

    let t_out = STime::now();
    let t_in = t_out.earlier(duration);
    write!(ws, "{}\n  -{}\n", t_in, t_out)?;

    print_result(fname, &ws)
}

pub fn print_result(fname: &Option<String>, s: &str) -> anyhow::Result<()> {
    match &fname {
        Some(nm) => {
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .append(true)
                .open(nm)?;
            std::io::Write::write_fmt(&mut f, format_args!("{}\n", s))?;
        }
        None => {
            println!("{}", s);
        }
    }
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
