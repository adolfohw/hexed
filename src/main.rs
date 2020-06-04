use hexed::Hexed;
use structopt::StructOpt;

fn main() {
    if let Err(e) = Hexed::from_args().run() {
        eprintln!("{}", e);
    }
}
