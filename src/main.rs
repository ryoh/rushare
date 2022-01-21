use std::ffi::CString;
use std::process::exit;
use std::env;

use anyhow::{Result, Context};
use nix::libc::{EXIT_FAILURE};
use nix::sched::{CloneFlags, unshare};
use nix::unistd::{execvp, fork, ForkResult};
use nix::sys::signal::{self, signal, kill};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::mount::{mount, MsFlags};
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

    #[structopt(long, help = "mount proc filesystem first (implies --mount)")]
    mount_proc: bool,

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

    if opt.mount_proc {
        unshare_flags |= CloneFlags::CLONE_NEWNS;
    }

    // To immutable
    let unshare_flags = unshare_flags;

    // command building
    let path = match opt.prog {
        // Case no value
        None => env::var("SHELL").unwrap_or("/bin/sh".to_string()),
        // Case program value exists
        Some(prog) => prog,
    };
    let path = CString::new(path).expect("CString::new error");

    let mut argv: Vec<CString> = opt.args.iter()
        .map(|s| CString::new(s.as_str()).expect("CString::new error"))
        .collect();
    argv.insert(0, path.clone());


    // unshare and run command
    unsafe { signal(signal::SIGCHLD, signal::SigHandler::SigDfl) }?;

    // unshare
    unshare(unshare_flags).with_context(|| format!("{}: unshare failed", progname))?;

    // fork
    if opt.fork {
        unsafe{ signal(signal::SIGINT, signal::SigHandler::SigIgn) }?;
        unsafe{ signal(signal::SIGTERM, signal::SigHandler::SigIgn) }?;

        match unsafe{ fork().with_context(|| format!("{}: fork failed", progname))? } {
            ForkResult::Parent { child } => {
                let status = waitpid(child, None).with_context(|| format!("{}: waitpid failed", progname))?;

                unsafe{ signal(signal::SIGINT, signal::SigHandler::SigDfl) }?;
                unsafe{ signal(signal::SIGTERM, signal::SigHandler::SigDfl) }?;

                match status {
                    WaitStatus::Exited(_pid, status) => {
                        exit(status);
                    }
                    WaitStatus::Signaled(pid, signal, _) => {
                        kill(pid, signal)?;
                    }
                    _ => {
                        eprintln!("child exit failed");
                    }
                }
                exit(EXIT_FAILURE);
            }
            ForkResult::Child => {
                println!("forked.");
            }
        }
    }

    if opt.mount {
        // change propagation
        mount::<str, str, str, str>(None, "/", None, MsFlags::MS_PRIVATE | MsFlags::MS_REC, None)
            .with_context(|| format!("{}: change propagation failed", progname))?;
    }

    if opt.mount_proc {
        // proc mount
        mount::<str, str, str, str>(Some("proc"), "/proc", Some("proc"), MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC | MsFlags::MS_NODEV | MsFlags::MS_NOATIME, None)
            .with_context(|| format!("{}: proc remount failed", progname))?;
    }

    // execvp
    execvp(&path, &argv).with_context(|| format!("{}: execvp failed", progname))?;
    

    Ok(())
}
