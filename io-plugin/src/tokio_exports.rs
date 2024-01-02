use tokio::io::BufReader;

pub type Stdin = BufReader<tokio::io::Stdin>;

pub fn stdin() -> Stdin {
    BufReader::new(tokio::io::stdin())
}

pub type Stdout = tokio::io::Stdout;

pub fn stdout() -> Stdout {
    tokio::io::stdout()
}