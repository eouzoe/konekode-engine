use std::io::{self, BufRead, Write};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::BufWriter::new(io::stdout().lock());
    let mut ui = konekode_probe::new_ui();
    for line in stdin.lock().lines() {
        writeln!(stdout, "{}", konekode_probe::reply(&mut ui, &line?))?;
    }
    stdout.flush()
}
