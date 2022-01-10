use std::ffi::CString;

use anyhow::{Result, Context};
use std::env;
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

    #[structopt(short, long, help = "unshare UTS namespace (hostname etc)")]
    uts: bool,

    #[structopt(short, long, help = "unshare System V IPC namespace")]
    ipc: bool,

    #[structopt(short, long, help = "unshare network namespace")]
    net: bool,

    #[structopt(short, long, help = "unshare pid namespace")]
    pid: bool,

    #[structopt(short = "U", long, help = "unshare user namespace")]
    user: bool,

    #[structopt(short = "C", long, help = "unshare cgroup namespace")]
    cgroup: bool,

    #[structopt(short, long, help = "fork before launching <program>")]
    fork: bool,

    #[structopt(name="program")]
    prog: Option<String>,

    #[structopt(name="arguments")]
    args: Vec<String>,
}

fn main() -> Result<()> {
    // Get program name
    let progname = std::env::current_exe()?
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
    // Get commandline arguments
    let opt = Opt::from_args();
    let mut unshare_flags: CloneFlags = CloneFlags::empty();

    // Set unshare namespace
    if opt.mount {
        unshare_flags |= CloneFlags::CLONE_NEWNS;
    }

    if opt.uts {
        unshare_flags |= CloneFlags::CLONE_NEWUTS;
    }
    
    if opt.ipc {
        unshare_flags |= CloneFlags::CLONE_NEWIPC;
    }

    if opt.net {
        unshare_flags |= CloneFlags::CLONE_NEWNET;
    }

    if opt.pid {
        unshare_flags |= CloneFlags::CLONE_NEWPID;
    }

    if opt.user {
        unshare_flags |= CloneFlags::CLONE_NEWUSER;
    }

    if opt.cgroup {
        unshare_flags |= CloneFlags::CLONE_NEWCGROUP;
    }

    // To immutable
    let unshare_flags = unshare_flags;

    if opt.fork {
        println!("forked!!");
    }

    // command building
    let path = match opt.prog {
        // Case no value
        None => CString::new(env::var("SHELL").unwrap_or("/bin/sh".to_string()))?,
        // Case program value exists
        Some(prog) => CString::new(prog)?,
    };

    let mut argv: Vec<CString> = opt.args.iter()
        .map(|s| CString::new(s.as_str()).expect("CString::new error"))
        .collect();
    argv.insert(0, path.clone());


    // unshare and run command
    unsafe { signal(signal::SIGCHLD, signal::SigHandler::SigDfl) }?;

    // unshare
    unshare(unshare_flags).with_context(|| format!("{}: unshare failed", progname))?;

    // execvp
    execvp(&path, &argv).with_context(|| format!("{}: execvp failed", progname))?;
    

    Ok(())
}
