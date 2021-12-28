use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "rushare", about = "Run a program with some namespaces unshared from the parent.")]
struct Opt {
    #[structopt(short, long)]
    mount: bool
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
