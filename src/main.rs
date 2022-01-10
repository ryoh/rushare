use std::ffi::CString;

use anyhow::Result;
use nix::sched::{CloneFlags, unshare};
use nix::sys::signal::{self, signal};
use nix::unistd::execvp;
use structopt::{clap, StructOpt};

#[derive(Debug, StructOpt)]
#[structopt(name = "rushare")]
#[structopt(about = "Run a program with some namespaces unshared from the parent.")]
#[structopt(long_version(option_env!("LONG_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))))]
#[structopt(setting(clap::AppSettings::ColoredHelp))]
struct Opt {
    #[structopt(short, long, help = "unshare mounts namespace")]
    mount: bool,

    #[structopt(short, long, help = "fork before launching <program>")]
    fork: bool,

    #[structopt(name="program")]
    prog: String,

    #[structopt(name="arguments")]
    args: Vec<String>,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let mut unshare_flags: CloneFlags = CloneFlags::empty();

    // parse command arguments
    if opt.mount {
        unshare_flags |= CloneFlags::CLONE_NEWNS;
    }

    if opt.fork {
        println!("forked!!");
    }

    // command building
    let path = match opt.prog.as_str() {
        "" => CString::new("/bin/sh")?,
        _ => CString::new(opt.prog)?,
    };

    let mut argv: Vec<CString> = opt.args.iter()
        .map(|s| CString::new(s.as_str()).expect("CString::new error"))
        .collect();
    argv.insert(0, path.clone());


    // unshare and run command
    unsafe { signal(signal::SIGCHLD, signal::SigHandler::SigDfl) }?;

    unshare(unshare_flags)?;

    execvp(&path, &argv)?;
    

    Ok(())
}
