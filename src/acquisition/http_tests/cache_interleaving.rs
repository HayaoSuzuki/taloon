//! Replay scheduler checkpoints against real acquisition batches and a shared cache.
use super::*;
use crate::acquisition::cache::Checkpoint;
use serde::Deserialize;
use std::{
    path::PathBuf,
    sync::mpsc,
    thread::{self, JoinHandle},
    time::Duration,
};

const ARCHIVE_URL: &str = "https://files.test/shared.whl";
const DEADLINE: Duration = Duration::from_secs(5);

#[derive(Deserialize)]
struct Step {
    event: Event,
    visible: Vec<u8>,
}

#[derive(Deserialize)]
struct Event {
    kind: Operation,
    key: u8,
    observed: u8,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum Operation {
    Begin,
    Sync,
    Publish,
    Clear,
    Lookup,
    Open,
    FinishRead,
}

struct PausedBatch {
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
            checkpoint,
            resume,
            result,
        }
    }

    fn expect(&self, expected: Checkpoint) {
        assert_eq!(self.checkpoint.recv_timeout(DEADLINE).unwrap(), expected);
    }

    fn advance(&self, expected: Checkpoint) {
        self.resume.send(()).unwrap();
        self.expect(expected);
    }

    fn finish(self) -> u8 {
        drop(self.checkpoint);
        drop(self.resume);
        self.result.join().unwrap()
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
    root: PathBuf,
    archives: [Vec<u8>; 2],
    writers: [Option<PausedBatch>; 2],
    reader: Option<PausedBatch>,
}

impl Replay {
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
        let acquirer =
            Acquirer::with_transport(AcquisitionOptions::testing(&self.root, offline), transport)
                .unwrap();
        (acquirer, inspection)
    }

    fn lookup(&self, key: u8) -> u8 {
        let (acquirer, inspection) = self.acquisition(key, true);
        observe(&acquirer, &inspection, key, true)
    }

    fn apply(&mut self, event: &Event) {
        let key = event.key;
        match event.kind {
            Operation::Begin => {
                let (acquirer, inspection) = self.acquisition(key, false);
                let batch = PausedBatch::start(acquirer, inspection, key, false);
                batch.expect(Checkpoint::TemporaryCreated);
                assert!(self.writers[usize::from(key - 1)].replace(batch).is_none());
            }
            Operation::Sync => self.writers[usize::from(key - 1)]
                .as_ref()
                .unwrap()
                .advance(Checkpoint::BeforePublish),
            Operation::Publish => assert_eq!(
                self.writers[usize::from(key - 1)].take().unwrap().finish(),
                key
            ),
            Operation::Clear => {
                let cache = EvidenceCache::new(self.root.join("cache"));
                assert_eq!(cache.clear().unwrap(), u64::from(event.observed));
            }
            Operation::Lookup => assert_eq!(self.lookup(key), event.observed),
            Operation::Open => {
                let (acquirer, inspection) = self.acquisition(key, true);
                let batch = PausedBatch::start(acquirer, inspection, key, true);
                batch.expect(Checkpoint::ReaderOpened);
                assert!(self.reader.replace(batch).is_none());
            }
            Operation::FinishRead => {
                assert_eq!(self.reader.take().unwrap().finish(), event.observed);
            }
        }
    }

    fn finish(self) {
        for batch in self.writers.into_iter().flatten().chain(self.reader) {
            batch.finish();
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
    let steps: Vec<Step> = serde_json::from_str(line).unwrap();
    assert_eq!(steps.len(), 32);
    let temp = tempfile::tempdir().unwrap();
    let mut replay = Replay {
        root: temp.path().to_owned(),
        archives: ["first", "second"].map(|tag| {
            crate::testfixtures::wheel(&crate::testfixtures::metadata(&format!(
                "License-Expression: MIT\nX-Variant: {tag}"
            )))
        }),
        writers: [None, None],
        reader: None,
    };

    for (step, state) in steps.iter().enumerate() {
        eprintln!("Quint shared cache seed {seed}, step {}", step + 1);
        // Act
        replay.apply(&state.event);

        // Assert
        for key in [1, 2] {
            let expected = if state.visible.contains(&key) { key } else { 0 };
            assert_eq!(replay.lookup(key), expected, "where digest is {key}");
        }
    }
    replay.finish();
}
