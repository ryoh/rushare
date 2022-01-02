use anyhow::Result;
use nix::sched::CloneFlags;
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

    prog: String,

    args: Vec<String>,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let mut clone_flag: CloneFlags = CloneFlags::empty();

    if opt.mount {
        clone_flag |= CloneFlags::CLONE_NEWNS;
    }

    if opt.fork {
        println!("forked!!");
    }

    println!("{:?}, {:?}", opt, clone_flag);

    Ok(())
}
