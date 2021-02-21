use pad::{Alignment, PadStr};
use serde::Serialize;
use std::sync::{Arc, RwLock};

use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Display},
};

use lychee::{collector::Input, Response, Status::*};

// Maximum padding for each entry in the final statistics output
const MAX_PADDING: usize = 20;

#[derive(Serialize)]
struct Stats {
    total: usize,
    successful: usize,
    failures: usize,
    timeouts: usize,
    redirects: usize,
    excludes: usize,
    errors: usize,
    fail_map: HashMap<Input, HashSet<Response>>,
}

#[derive(Serialize)]
pub struct ResponseStats {
    inner: RwLock<Stats>,
}

impl ResponseStats {
    pub fn new() -> Arc<Self> {
        let fail_map = HashMap::new();
        Arc::new(ResponseStats {
            inner: RwLock::new(Stats {
                total: 0,
                successful: 0,
                failures: 0,
                timeouts: 0,
                redirects: 0,
                excludes: 0,
                errors: 0,
                fail_map,
            }),
        })
    }

    pub fn add(&mut self, response: Response) {
        self.inner.get_mut().unwrap().total += 1;
        match response.status {
            Failed(_) => self.inner.write().unwrap().failures += 1,
            Timeout(_) => self.inner.write().unwrap().timeouts += 1,
            Redirected(_) => self.inner.write().unwrap().redirects += 1,
            Excluded => self.inner.write().unwrap().excludes += 1,
            Error(_) => self.inner.write().unwrap().errors += 1,
            _ => self.inner.write().unwrap().successful += 1,
        }

        if matches!(
            response.status,
            Failed(_) | Timeout(_) | Redirected(_) | Error(_)
        ) {
            let fail = self
                .inner
                .write()
                .unwrap()
                .fail_map
                .entry(response.source.clone())
                .or_default();
            fail.insert(response);
        };
    }

    pub fn is_success(&self) -> bool {
        self.inner.read().unwrap().total
            == self.inner.read().unwrap().successful + self.inner.read().unwrap().excludes
    }
}

fn write_stat(f: &mut fmt::Formatter, title: &str, stat: usize) -> fmt::Result {
    let fill = title.chars().count();
    f.write_str(title)?;
    f.write_str(
        &stat
            .to_string()
            .pad(MAX_PADDING - fill, '.', Alignment::Right, false),
    )?;
    f.write_str("\n")
}

impl Display for ResponseStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let separator = "-".repeat(MAX_PADDING + 1);

        writeln!(f, "📝 Summary")?;
        writeln!(f, "{}", separator)?;
        write_stat(f, "🔍 Total", self.inner.read().unwrap().total)?;
        write_stat(f, "✅ Successful", self.inner.read().unwrap().successful)?;
        write_stat(f, "⏳ Timeouts", self.inner.read().unwrap().timeouts)?;
        write_stat(f, "🔀 Redirected", self.inner.read().unwrap().redirects)?;
        write_stat(f, "👻 Excluded", self.inner.read().unwrap().excludes)?;
        write_stat(
            f,
            "🚫 Errors",
            self.inner.read().unwrap().errors + self.inner.read().unwrap().failures,
        )?;

        if !&self.inner.read().unwrap().fail_map.is_empty() {
            writeln!(f)?;
        }
        for (input, responses) in &self.inner.read().unwrap().fail_map {
            writeln!(f, "Input: {}", input)?;
            for response in responses {
                writeln!(
                    f,
                    "   {} {}\n      {}",
                    response.status.icon(),
                    response.uri,
                    response.status
                )?
            }
        }
        writeln!(f)
    }
}

#[cfg(test)]
mod test_super {
    use lychee::{test_utils::website, Status};

    use super::*;

    #[test]
    fn test_stats() {
        let mut stats = ResponseStats::new();
        stats.add(Response {
            uri: website("http://example.org/ok"),
            status: Status::Ok(http::StatusCode::OK),
            source: Input::Stdin,
        });
        stats.add(Response {
            uri: website("http://example.org/failed"),
            status: Status::Failed(http::StatusCode::BAD_GATEWAY),
            source: Input::Stdin,
        });
        stats.add(Response {
            uri: website("http://example.org/redirect"),
            status: Status::Redirected(http::StatusCode::PERMANENT_REDIRECT),
            source: Input::Stdin,
        });
        let mut expected_map = HashMap::new();
        expected_map.insert(
            Input::Stdin,
            vec![
                Response {
                    uri: website("http://example.org/failed"),
                    status: Status::Failed(http::StatusCode::BAD_GATEWAY),
                    source: Input::Stdin,
                },
                Response {
                    uri: website("http://example.org/redirect"),
                    status: Status::Redirected(http::StatusCode::PERMANENT_REDIRECT),
                    source: Input::Stdin,
                },
            ]
            .into_iter()
            .collect::<HashSet<_>>(),
        );
        assert_eq!(stats.inner.read().unwrap().fail_map, expected_map);
    }
}
