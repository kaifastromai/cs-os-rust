extern crate getopts;
use crate::unistd::getcwd;
use ffi::{CStr, CString};
use getopts::Options;
use nix::{fcntl::OFlag, sys::stat::Mode, unistd::chdir};
use nix::{sys::wait::WaitPidFlag, sys::wait::WaitStatus, unistd::close};
use std::str::FromStr;
use std::{env, path::Path};
use std::{ffi, io::BufRead};
static DEV: bool = true;
use nix::unistd::{self, dup2, fork, pipe};

fn main() {
    let env_args: Vec<String> = env::args().collect();
    let prog = env_args[0].clone();

    let mut opts = Options::new();

    opts.optopt(
        "d",
        "dir",
        "Specifies the path that will be used as the working directory of the process",
        "DIR",
    );

    opts.optopt(
        "i",
        "input",
        "Specifies a file to read as input instead of stdin",
        "INPUT",
    );
    opts.optopt(
        "o",
        "",
        "Specifies a file (in overwrite mode) to output to instead of stdout.",
        "OUT",
    );
    opts.optopt(
        "a",
        "",
        "Specifies a file (in add mode) to output to instead of stdout.",
        "OUT",
    );
    opts.reqopt(
        "1",
        "exec1",
        "The process to execute. This option is required.",
        "EXEC1",
    );
    opts.optopt("2", "exec2", "The second process to execute.", "EXEC2");

    opts.optflag("h", "help", "Print program help.");

    let matches = match opts.parse(&env_args[1..]) {
        Ok(m) => m,
        Err(f) => {
            print!(
                "{}. \n{}",
                f.to_string(),
                opts.usage("Please check the usage: ")
            );
            return;
        }
    };

    let mut args: Args = Args::default();

    if matches.opt_present("h") {
        print!("{}", opts.usage("Usage:"));
    }

    env_args.into_iter().for_each(|s| match s.as_str() {
        "-o" => {
            args.outfile = match matches.opt_str("o") {
                Some(sk) => sk,
                None => String::new(),
            };
            args.outfile_type = String::from_str("o").unwrap();
        }
        "-a" => {
            args.outfile = match matches.opt_str("a") {
                Some(sk) => sk,
                None => String::new(),
            };
            args.outfile_type = String::from_str("a").unwrap();
        }
        "-d" => {
            args.dir = match matches.opt_str("d") {
                Some(sk) => sk,
                None => String::new(),
            };
        }
        "-i" => {
            args.input = match matches.opt_str("i") {
                Some(sk) => sk,
                None => String::new(),
            };
        }
        "-1" => {
            args.exec1 = match matches.opt_str("1") {
                Some(sk) => sk,
                None => String::new(),
            };
        }
        "-2" => {
            args.exec2 = match matches.opt_str("2") {
                Some(sk) => sk,
                None => String::new(),
            };
        }

        _ => {}
    });

    if args.dir.is_empty() {
        args.dir = getcwd().unwrap().to_string_lossy().to_string();
    } else {
    }

    let fd1 = pipe().unwrap();
    let child1_id = fork();

    let oflg = OFlag::O_RDONLY;
    let mode = Mode::S_IRUSR;

    if matches.opt_present("i") {
        let infile = nix::fcntl::open(Path::new(&args.input), oflg | OFlag::O_CREAT, mode).unwrap();
        dup2(infile, nix::libc::STDIN_FILENO).unwrap();
    }
    match child1_id.unwrap() {
        unistd::ForkResult::Parent { child } => {
            let wait = nix::sys::wait::waitpid(child, None).unwrap();
            match wait {
                WaitStatus::Exited(p, excode) => {
                    println!("Child1 (process {}) exited with code {}", p, excode)
                }
                WaitStatus::Signaled(_, _, _) => {}
                WaitStatus::Stopped(_, _) => {}
                WaitStatus::PtraceEvent(_, _, _) => {}
                WaitStatus::PtraceSyscall(_) => {}
                WaitStatus::Continued(_) => {}
                WaitStatus::StillAlive => {}
            }
        }
        unistd::ForkResult::Child => {
            if (matches.opt_present("exec2")) {
                dup2(fd1.1, nix::libc::STDIN_FILENO).unwrap();
            }
            if matches.opt_present("o") || matches.opt_present("a") {
                let oflg = if args.outfile_type == "o" {
                    OFlag::O_WRONLY | OFlag::O_CREAT | OFlag::O_TRUNC
                } else {
                    OFlag::O_WRONLY | OFlag::O_APPEND | OFlag::O_CREAT
                };
                let outfile =
                    nix::fcntl::open(Path::new(&args.outfile), oflg, Mode::S_IRUSR).unwrap();
                dup2(outfile, nix::libc::STDOUT_FILENO).unwrap();
            }

            nix::unistd::execvp(
                &CString::new(args.exec1.clone()).unwrap(),
                &[&CString::new(args.exec1).unwrap()],
            )
            .unwrap();
        }
    }
}
#[derive(Default, Debug)]
struct Args {
    dir: String,
    input: String,
    outfile: String,
    outfile_type: String,
    exec1: String,
    exec2: String,
}
