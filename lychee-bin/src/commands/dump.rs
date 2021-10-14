use anyhow::Result;
use std::{
    collections::HashSet,
    io::{self, Write},
};
use tokio_stream::StreamExt;

use lychee_lib::{Client, Request};

use crate::ExitCode;

/// Dump all detected links to stdout without checking them
pub(crate) async fn dump<'a, S>(client: Client, links: S) -> Result<ExitCode>
where
    S: futures::Stream<Item = Result<HashSet<Request>>>,
{
    println!("dump");
    let mut stdout = io::stdout();
    tokio::pin!(links);
    while let Some(links) = links.next().await {
        println!("link batch");
        for link in links? {
            if client.filtered(&link.uri) {
                continue;
            }
            // Avoid panic on broken pipe.
            // See https://github.com/rust-lang/rust/issues/46016
            // This can occur when piping the output of lychee
            // to another program like `grep`.
            if let Err(e) = writeln!(stdout, "{}", &link) {
                if e.kind() != io::ErrorKind::BrokenPipe {
                    eprintln!("{}", e);
                    return Ok(ExitCode::UnexpectedFailure);
                }
            }
        }
    }
    Ok(ExitCode::Success)
}
