//! Replay scheduler checkpoints against real acquisition batches and a shared cache.
use super::*;
use crate::acquisition::cache::Checkpoint;
use quint_connect::{Driver, State, runner};
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    sync::mpsc,
    thread::{self, JoinHandle},
    time::Duration,
};

const ARCHIVE_URL: &str = "https://files.test/shared.whl";
const DEADLINE: Duration = Duration::from_secs(5);

#[derive(Debug, Deserialize, PartialEq)]
struct CacheState {
    #[serde(alias = "SharedCache::SharedCacheModel::event")]
    event: Event,
    #[serde(alias = "SharedCache::SharedCacheModel::visible")]
    visible: BTreeSet<u8>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
struct Event {
    kind: Operation,
    key: u8,
    observed: u8,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
enum Operation {
    Init,
    Begin,
    Sync,
    Publish,
    Clear,
    Lookup,
    Open,
    FinishRead,
}

struct PausedBatch {
    key: u8,
    synced: bool,
    checkpoint: mpsc::Receiver<Checkpoint>,
    resume: mpsc::Sender<()>,
    result: JoinHandle<u8>,
}

impl PausedBatch {
    fn start(mut acquirer: Acquirer, inspection: Inspection, key: u8, offline: bool) -> Self {
        let (ready, checkpoint) = mpsc::channel();
        let (resume, release) = mpsc::channel();
        let release = Mutex::new(release);
        acquirer.cache.checkpoint = Some(Box::new(move |point| {
            if ready.send(point).is_ok() {
                release
                    .lock()
                    .unwrap()
                    .recv_timeout(DEADLINE)
                    .unwrap_or_else(|error| {
                        assert_eq!(error, mpsc::RecvTimeoutError::Disconnected);
                    });
            }
        }));
        let result = thread::spawn(move || observe(&acquirer, &inspection, key, offline));
        Self {
            key,
            synced: false,
            checkpoint,
            resume,
            result,
        }
    }

    fn expect(&self, expected: Checkpoint) {
        assert_eq!(self.checkpoint.recv_timeout(DEADLINE).unwrap(), expected);
    }

    fn advance(&mut self, expected: Checkpoint) {
        self.resume.send(()).unwrap();
        self.expect(expected);
        self.synced = true;
    }

    fn finish(self) -> u8 {
        self.join().unwrap()
    }

    fn join(self) -> thread::Result<u8> {
        drop(self.checkpoint);
        drop(self.resume);
        self.result.join()
    }
}

fn observe(acquirer: &Acquirer, inspection: &Inspection, key: u8, offline: bool) -> u8 {
    let batch = acquirer.acquire(inspection.requests());
    assert!(batch.warnings.is_empty());
    if offline {
        assert_eq!(batch.summary.network_requests, 0);
    }
    let report = inspection.evaluate(batch.results);
    let artifact = &report.packages()[0].artifacts[0];
    if let Some(evidence) = &artifact.evidence {
        assert_eq!(report.exit_code(false), 0);
        assert_eq!(Some(evidence.content_hash), artifact.locked_hash);
        assert_eq!(evidence.strength, EvidenceRequirement::ArchiveVerified);
        key
    } else {
        assert_eq!(report.exit_code(false), 2);
        0
    }
}

struct Replay {
    root: tempfile::TempDir,
    archives: [Vec<u8>; 2],
    writers: [Option<PausedBatch>; 2],
    reader: Option<PausedBatch>,
}

impl Replay {
    fn new(root: tempfile::TempDir, variant: &str) -> Self {
        Self {
            root,
            archives: ["first", "second"].map(|tag| {
                crate::testfixtures::wheel(&crate::testfixtures::metadata(&format!(
                    "License-Expression: MIT\nX-Variant: {tag}{variant}"
                )))
            }),
            writers: [None, None],
            reader: None,
        }
    }

    fn visible(&self) -> BTreeSet<u8> {
        [1, 2]
            .into_iter()
            .filter(|&key| self.lookup(key) == key)
            .collect()
    }

    fn acquisition(&self, key: u8, offline: bool) -> (Acquirer, Inspection) {
        let bytes = &self.archives[usize::from(key - 1)];
        let digest = ContentHash::digest(bytes);
        let lock = format!(
            "version=1\nrevision=5\n[[package]]\nname='example'\nversion='1.0'\n\
             source={{registry='https://example.test/simple'}}\n\
             wheels=[{{url='{ARCHIVE_URL}',hash='{digest}'}}]"
        );
        let inspection = Inspection::from_documents(
            &lock,
            "[tool.taloon]\nschema-version=1\nallow=['MIT']",
            EvaluationDate::parse("2026-10-31").unwrap(),
        )
        .unwrap();
        let transport = Fixture::default();
        transport.add(ARCHIVE_URL, 200, bytes);
        let acquirer = Acquirer::with_transport(
            AcquisitionOptions::testing(self.root.path(), offline),
            transport,
        )
        .unwrap();
        (acquirer, inspection)
    }

    fn lookup(&self, key: u8) -> u8 {
        let (acquirer, inspection) = self.acquisition(key, true);
        observe(&acquirer, &inspection, key, true)
    }

    fn apply(&mut self, kind: Operation, key: u8) -> u8 {
        match kind {
            Operation::Init => 0,
            Operation::Begin => {
                let (acquirer, inspection) = self.acquisition(key, false);
                let batch = PausedBatch::start(acquirer, inspection, key, false);
                batch.expect(Checkpoint::TemporaryCreated);
                assert!(self.writers[usize::from(key - 1)].replace(batch).is_none());
                0
            }
            Operation::Sync => {
                self.writers[usize::from(key - 1)]
                    .as_mut()
                    .unwrap()
                    .advance(Checkpoint::BeforePublish);
                0
            }
            Operation::Publish => {
                assert_eq!(
                    self.writers[usize::from(key - 1)].take().unwrap().finish(),
                    key
                );
                0
            }
            Operation::Clear => {
                let cache = EvidenceCache::new(self.root.path().join("cache"));
                u8::try_from(cache.clear().unwrap()).unwrap()
            }
            Operation::Lookup => self.lookup(key),
            Operation::Open => {
                let (acquirer, inspection) = self.acquisition(key, true);
                let batch = PausedBatch::start(acquirer, inspection, key, true);
                batch.expect(Checkpoint::ReaderOpened);
                assert!(self.reader.replace(batch).is_none());
                0
            }
            Operation::FinishRead => self.reader.take().unwrap().finish(),
        }
    }
}

impl Drop for Replay {
    fn drop(&mut self) {
        for batch in self
            .writers
            .iter_mut()
            .filter_map(Option::take)
            .chain(self.reader.take())
        {
            let result = batch.join();
            if !thread::panicking() {
                result.unwrap();
            }
        }
    }
}

#[rstest::rstest]
fn quint_replays_shared_disk_cache(#[values(1, 2, 3, 4, 5, 6, 7, 8)] seed: usize) {
    // Arrange
    let line = include_str!("../../../tests/fixtures/oracle/shared-cache.jsonl")
        .lines()
        .nth(seed - 1)
        .unwrap();
    let steps: Vec<CacheState> = serde_json::from_str(line).unwrap();
    assert_eq!(steps.len(), 32);
    let temp = tempfile::tempdir().unwrap();
    let mut replay = Replay::new(temp, "");

    for (step, state) in steps.iter().enumerate() {
        eprintln!("Quint shared cache seed {seed}, step {}", step + 1);
        // Act
        let observed = replay.apply(state.event.kind, state.event.key);

        // Assert
        assert_eq!(observed, state.event.observed);
        assert_eq!(replay.visible(), state.visible);
    }
}

struct CacheDriver {
    replay: Replay,
    event: Event,
}

impl State<CacheDriver> for CacheState {
    fn from_driver(driver: &CacheDriver) -> quint_connect::Result<Self> {
        Ok(Self {
            event: driver.event,
            visible: driver.replay.visible(),
        })
    }
}

impl Driver for CacheDriver {
    type State = CacheState;

    fn step(&mut self, step: &quint_connect::Step) -> quint_connect::Result {
        quint_connect::switch!(step {
            init => self.apply(Operation::Init, 0),
            write(key: u8) => self.write(key),
            clear => self.apply(Operation::Clear, 0),
            lookup(key: u8) => self.apply(Operation::Lookup, key),
            open(key: u8) => self.apply(Operation::Open, key),
            finishRead => self.finish_read(),
        })
    }
}

impl CacheDriver {
    fn write(&mut self, key: u8) {
        let kind = match &self.replay.writers[usize::from(key - 1)] {
            None => Operation::Begin,
            Some(batch) if batch.synced => Operation::Publish,
            Some(_) => Operation::Sync,
        };
        self.apply(kind, key);
    }

    fn finish_read(&mut self) {
        let key = self.replay.reader.as_ref().unwrap().key;
        self.apply(Operation::FinishRead, key);
    }

    fn apply(&mut self, kind: Operation, key: u8) {
        let observed = self.replay.apply(kind, key);
        self.event = Event {
            kind,
            key,
            observed,
        };
    }
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(16))]
    #[test]
    #[ignore = "requires the pinned Quint CLI in PATH"]
    fn quint_connect_shared_disk_cache(
        seed in 1..=u32::MAX,
        variant in "[a-z0-9]{0,48}",
    ) {
        // Arrange
        let driver = CacheDriver {
            replay: Replay::new(tempfile::tempdir().unwrap(), &variant),
            event: Event { kind: Operation::Init, key: 0, observed: 0 },
        };
        // Act
        let result = runner::run_test(driver, runner::Config {
            test_name: format!("shared disk cache, seed {seed}"),
            gen_config: runner::RunConfig {
                spec: "formal/quint/shared_cache.qnt".into(),
                main: Some("SharedCache".into()),
                init: None,
                step: None,
                max_samples: Some(1),
                max_steps: Some(32),
                seed: seed.to_string(),
            },
        });

        // Assert
        proptest::prop_assert!(result.is_ok(), "{result:#?}");
    }
}
