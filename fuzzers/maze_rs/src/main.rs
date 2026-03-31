#[cfg(windows)]
use std::ptr::write_volatile;
use std::{borrow::Cow, path::PathBuf};

#[cfg(not(feature = "tui"))]
use libafl::monitors::SimpleMonitor;
#[cfg(feature = "tui")]
use libafl::monitors::tui::TuiMonitor;
use libafl::{
    Evaluator,
    corpus::{CorpusId, InMemoryCorpus, OnDiskCorpus},
    events::SimpleEventManager,
    executors::{ExitKind, inprocess::InProcessExecutor},
    feedbacks::{CrashFeedback, MaxMapFeedback},
    fuzzer::{Fuzzer, StdFuzzer},
    inputs::{BytesInput, HasTargetBytes},
    mutators::{MutationResult, Mutator},
    observers::StdMapObserver,
    schedulers::QueueScheduler,
    stages::mutational::StdMutationalStage,
    state::{HasRand, StdState},
};
use libafl_bolts::{
    AsSlice, HasLen, Named,
    rands::{Rand, StdRand},
    tuples::tuple_list,
};
use libafl_ijon::{MAP_PTR, MAP_SIZE, ijon_zero_map};

mod maze;

/// Custom mutator for only generating from the `abcd` options
pub struct AppendMutator;

impl AppendMutator {
    pub fn new() -> Self {
        Self
    }
}

impl Named for AppendMutator {
    fn name(&self) -> &Cow<'static, str> {
        &Cow::Borrowed("AppendMutator")
    }
}

impl<I, S> Mutator<I, S> for AppendMutator
where
    I: HasLen + AsMut<Vec<u8>>,
    S: HasRand,
{
    fn mutate(&mut self, state: &mut S, input: &mut I) -> Result<MutationResult, libafl::Error> {
        let num_additions = state.rand_mut().choose(1..=3).unwrap_or(1);

        let bytes = input.as_mut();

        let moves = [b'a', b'b', b'c', b'd'];

        for _ in 0..num_additions {
            let random_move = *state.rand_mut().choose(&moves).unwrap_or(&b'a');
            bytes.push(random_move);
        }

        Ok(MutationResult::Mutated)
    }

    fn post_exec(
        &mut self,
        _state: &mut S,
        _corpus_id: Option<CorpusId>,
    ) -> Result<(), libafl::Error> {
        Ok(())
    }
}

pub fn main() {
    // The closure that we want to fuzz
    let mut harness = |input: &BytesInput| {
        let target = input.target_bytes();
        let buf = target.as_slice();

        unsafe { ijon_zero_map() };

        maze::maze(buf);

        ExitKind::Ok
    };

    // Create an observation channel using the signals map
    let observer = unsafe { StdMapObserver::from_mut_ptr("ijon_map", MAP_PTR, MAP_SIZE) };

    // Feedback to rate the interestingness of an input, obtained by ANDing the interestingness of both feedbacks
    let mut feedback = MaxMapFeedback::new(&observer);

    // A feedback to choose if an input is a solution or not
    let mut objective = CrashFeedback::new();

    // create a State from scratch
    let mut state = StdState::new(
        // RNG
        StdRand::new(),
        // Corpus that will be evolved, we keep it in memory for performance
        InMemoryCorpus::new(),
        // Corpus in which we store solutions (crashes in this example),
        // on disk so the user can get them after stopping the fuzzer
        OnDiskCorpus::new(PathBuf::from("./crashes")).unwrap(),
        // States of the feedbacks.
        // The feedbacks can report the data that should persist in the State.
        &mut feedback,
        // Same for objective feedbacks
        &mut objective,
    )
    .unwrap();

    // The Monitor trait define how the fuzzer stats are displayed to the user
    #[cfg(not(feature = "tui"))]
    let mon = SimpleMonitor::new(|s| println!("{s}"));
    #[cfg(feature = "tui")]
    let mon = TuiMonitor::builder()
        .title("InProcess Maze Fuzzer")
        .enhanced_graphics(true)
        .build();

    // The event manager handle the various events generated during the fuzzing loop
    // such as the notification of the addition of a new item to the corpus
    let mut manager = SimpleEventManager::new(mon);

    // A queue policy to get testcasess from the corpus
    let scheduler = QueueScheduler::new();

    // A fuzzer with feedbacks and a corpus scheduler
    let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

    // Create the executor for an in-process function with just one observer
    let mut executor = InProcessExecutor::new(
        &mut harness,
        tuple_list!(observer),
        &mut fuzzer,
        &mut state,
        &mut manager,
    )
    .expect("Failed to create the Executor");

    if state.must_load_initial_inputs() {
        let msg = BytesInput::new(b"a".to_vec());
        fuzzer
            .evaluate_input(&mut state, &mut executor, &mut manager, &msg)
            .unwrap();
    }

    // Setup a mutational stage with a the custom mutator
    let mutator = AppendMutator::new();
    let mut stages = tuple_list!(StdMutationalStage::new(mutator));

    fuzzer
        .fuzz_loop(&mut stages, &mut executor, &mut state, &mut manager)
        .expect("Error in the fuzzing loop");
}
