use anyhow::Context;
pub use slog::o;
pub use slog_scope::{debug, error, info, logger, trace, warn};
pub use slog_scope_futures::FutureExt;

use chrono::format::{Fixed, Item, Numeric, Pad};
use slog::{Drain, Duplicate, LevelFilter, Logger};
use slog_async::Async;
use slog_scope::GlobalLoggerGuard;
use slog_term::{FullFormat, PlainDecorator, TermDecorator};
use std::fs::OpenOptions;

use crate::config::Config;

pub fn init(config: &Config) -> anyhow::Result<GlobalLoggerGuard> {
    let term_drain = config.log.term.then(|| {
        let decorator = TermDecorator::new().build();

        FullFormat::new(decorator)
            .use_custom_timestamp(local_timestamp)
            .build()
    });

    let file_drain = if let Some(ref log_file) = config.log.file {
        anyhow::ensure!(
            !log_file.exists() && log_file.is_file(),
            "Log file must be a file"
        );

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(log_file)
            .context("Failed to create logger")?;

        let decorator = PlainDecorator::new(file);
        Some(
            FullFormat::new(decorator)
                .use_custom_timestamp(local_timestamp)
                .build(),
        )
    } else {
        None
    };

    let drain = LevelFilter(
        Duplicate(Optional(term_drain), Optional(file_drain)),
        config.log.level,
    )
    .fuse();
    let drain = Async::new(drain).build().fuse();

    let logger = Logger::root(drain, o!());
    let guard = slog_scope::set_global_logger(logger);

    Ok(guard)
}

const TIMESTAMP_FORMAT_ITEMS: &[Item<'static>] = &[
    Item::Fixed(Fixed::ShortMonthName),
    Item::Space(" "),
    Item::Numeric(Numeric::Day, Pad::Zero),
    Item::Space(" "),
    Item::Numeric(Numeric::Hour, Pad::Zero),
    Item::Literal(":"),
    Item::Numeric(Numeric::Minute, Pad::Zero),
    Item::Literal(":"),
    Item::Numeric(Numeric::Second, Pad::Zero),
    Item::Fixed(Fixed::Nanosecond3),
];

fn local_timestamp(io: &mut dyn std::io::Write) -> std::io::Result<()> {
    write!(
        io,
        "{}",
        chrono::Local::now().format_with_items(TIMESTAMP_FORMAT_ITEMS.iter()),
    )
}

struct Optional<D: Drain<Ok: Default>>(Option<D>);

impl<D: Drain<Ok: Default>> Drain for Optional<D> {
    type Ok = D::Ok;
    type Err = D::Err;

    fn log(
        &self,
        record: &slog::Record,
        values: &slog::OwnedKVList,
    ) -> std::result::Result<Self::Ok, Self::Err> {
        match self.0 {
            Some(ref drain) => drain.log(record, values),
            None => Ok(Self::Ok::default()),
        }
    }
}
