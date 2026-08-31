//! Native transaction-cost benchmark for the deterministic core.
//!
//! This intentionally uses only the standard library. It reports allocations
//! as well as elapsed time so the rollback backstop has a reproducible baseline
//! without adding a benchmark dependency to the simulation crate.

use drl_core::{Game, Tile};
use drl_protocol::{Command, Direction, ItemSpawnKind, Position};
use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const BENCHMARK_NAME: &str = "drl-core-transaction";
const SCHEMA_VERSION: u32 = 1;
const DEFAULT_ITERATIONS: usize = 100_000;
const DEFAULT_SAMPLES: usize = 5;
const DEFAULT_WARMUP: usize = 10_000;
const BENCH_SEED: u64 = 42;
const CONTRACT_ITERATIONS: usize = 1_000;
const CONTRACT_SAMPLES: usize = 3;
const CONTRACT_WARMUP: usize = 100;
const FIXTURE_WIDTH: u32 = 20;
const FIXTURE_HEIGHT: u32 = 15;

static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static DEALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static ALLOCATED_BYTES: AtomicU64 = AtomicU64::new(0);
static DEALLOCATED_BYTES: AtomicU64 = AtomicU64::new(0);
static COUNT_ALLOCATIONS: AtomicBool = AtomicBool::new(false);

struct CountingAllocator;

#[global_allocator]
static GLOBAL_ALLOCATOR: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    // SAFETY: Delegating the exact layout to the system allocator is valid.
    let pointer = unsafe { System.alloc(layout) };
    if !pointer.is_null() && COUNT_ALLOCATIONS.load(Ordering::Relaxed) {
      ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
      ALLOCATED_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
    }
    pointer
  }

  unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
    // SAFETY: Delegating the exact layout to the system allocator is valid.
    let pointer = unsafe { System.alloc_zeroed(layout) };
    if !pointer.is_null() && COUNT_ALLOCATIONS.load(Ordering::Relaxed) {
      ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
      ALLOCATED_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
    }
    pointer
  }

  unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
    // SAFETY: The pointer/layout pair was returned by this allocator.
    unsafe { System.dealloc(pointer, layout) };
    if COUNT_ALLOCATIONS.load(Ordering::Relaxed) {
      DEALLOCATIONS.fetch_add(1, Ordering::Relaxed);
      DEALLOCATED_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
    }
  }

  unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
    // SAFETY: The pointer/layout pair was returned by this allocator.
    let new_pointer = unsafe { System.realloc(pointer, layout, new_size) };
    if !new_pointer.is_null() && COUNT_ALLOCATIONS.load(Ordering::Relaxed) {
      ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
      ALLOCATED_BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
      DEALLOCATIONS.fetch_add(1, Ordering::Relaxed);
      DEALLOCATED_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
    }
    new_pointer
  }
}

#[derive(Clone, Copy)]
struct BenchConfig {
  iterations: usize,
  samples: usize,
  warmup: usize,
}

impl Default for BenchConfig {
  fn default() -> Self {
    Self {
      iterations: DEFAULT_ITERATIONS,
      samples: DEFAULT_SAMPLES,
      warmup: DEFAULT_WARMUP,
    }
  }
}

impl BenchConfig {
  fn parse() -> Self {
    let mut config = Self::default();
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let contract = args.iter().any(|arg| arg == "--contract");
    let mut index = 0;
    while index < args.len() {
      let flag = &args[index];
      let value = args.get(index + 1);
      match flag.as_str() {
        "--contract" => {}
        // Cargo forwards its target selector to custom benchmark binaries.
        "--bench" => {}
        "--iterations" => config.iterations = parse_positive(value, flag),
        "--samples" => config.samples = parse_positive(value, flag),
        "--warmup" => config.warmup = parse_nonnegative(value, flag),
        "--help" => {
          println!("usage: transaction [--contract] [--iterations N] [--samples N] [--warmup N]");
          std::process::exit(0);
        }
        _ => panic!("unknown transaction benchmark argument: {flag}"),
      }
      let consumes_value = matches!(flag.as_str(), "--iterations" | "--samples" | "--warmup")
        || (flag == "--bench" && value.is_some_and(|item| !item.starts_with("--")));
      index += if consumes_value { 2 } else { 1 };
    }

    if contract {
      Self {
        iterations: CONTRACT_ITERATIONS,
        samples: CONTRACT_SAMPLES,
        warmup: CONTRACT_WARMUP,
      }
    } else {
      config
    }
  }
}

fn parse_positive(value: Option<&String>, flag: &str) -> usize {
  let parsed = value
    .unwrap_or_else(|| panic!("missing value for {flag}"))
    .parse::<usize>()
    .unwrap_or_else(|_| panic!("invalid value for {flag}"));
  assert!(parsed > 0, "{flag} must be greater than zero");
  parsed
}

fn parse_nonnegative(value: Option<&String>, flag: &str) -> usize {
  value
    .unwrap_or_else(|| panic!("missing value for {flag}"))
    .parse::<usize>()
    .unwrap_or_else(|_| panic!("invalid value for {flag}"))
}

#[derive(Clone, Copy)]
struct AllocationSnapshot {
  allocations: u64,
  deallocations: u64,
  allocated_bytes: u64,
  deallocated_bytes: u64,
}

impl AllocationSnapshot {
  fn reset() {
    ALLOCATIONS.store(0, Ordering::Relaxed);
    DEALLOCATIONS.store(0, Ordering::Relaxed);
    ALLOCATED_BYTES.store(0, Ordering::Relaxed);
    DEALLOCATED_BYTES.store(0, Ordering::Relaxed);
  }

  fn read() -> Self {
    Self {
      allocations: ALLOCATIONS.load(Ordering::Relaxed),
      deallocations: DEALLOCATIONS.load(Ordering::Relaxed),
      allocated_bytes: ALLOCATED_BYTES.load(Ordering::Relaxed),
      deallocated_bytes: DEALLOCATED_BYTES.load(Ordering::Relaxed),
    }
  }
}

#[derive(Clone, Copy)]
struct Measurement {
  elapsed_ns: u128,
  allocations: u64,
  deallocations: u64,
  allocated_bytes: u64,
  deallocated_bytes: u64,
}

fn measure<F>(create_game: fn() -> Game, config: BenchConfig, mut workload: F) -> Measurement
where
  F: FnMut(&mut Game, usize),
{
  COUNT_ALLOCATIONS.store(false, Ordering::Relaxed);
  let mut timed_game = create_game();
  workload(&mut timed_game, config.warmup);
  let started = Instant::now();
  workload(&mut timed_game, config.iterations);
  let elapsed_ns = started.elapsed().as_nanos();
  black_box(timed_game);

  let mut allocation_game = create_game();
  workload(&mut allocation_game, config.warmup);
  AllocationSnapshot::reset();
  COUNT_ALLOCATIONS.store(true, Ordering::Relaxed);
  workload(&mut allocation_game, config.iterations);
  COUNT_ALLOCATIONS.store(false, Ordering::Relaxed);
  let allocations = AllocationSnapshot::read();
  black_box(allocation_game);
  Measurement {
    elapsed_ns,
    allocations: allocations.allocations,
    deallocations: allocations.deallocations,
    allocated_bytes: allocations.allocated_bytes,
    deallocated_bytes: allocations.deallocated_bytes,
  }
}

fn accepted_wait(game: &mut Game, iterations: usize) {
  for _ in 0..iterations {
    let result = game
      .step(Command::Wait)
      .expect("fixed wait must be accepted");
    let _ = black_box(result);
  }
}

fn accepted_move(game: &mut Game, iterations: usize) {
  for index in 0..iterations {
    let direction = if index.is_multiple_of(2) {
      Direction::East
    } else {
      Direction::West
    };
    let result = game
      .step(Command::Move(direction))
      .expect("fixed move must be accepted");
    let _ = black_box(result);
  }
}

fn rejected_blocked_move(game: &mut Game, iterations: usize) {
  for _ in 0..iterations {
    let result = game.step(Command::Move(Direction::West));
    assert!(result.is_err(), "fixed blocked move must be rejected");
    let _ = black_box(result);
  }
}

fn rejected_out_of_bounds_ranged(game: &mut Game, iterations: usize) {
  for _ in 0..iterations {
    let result = game.step(Command::AttackRanged(Position::new(-1, 1)));
    assert!(
      result.is_err(),
      "fixed out-of-bounds attack must be rejected"
    );
    let _ = black_box(result);
  }
}

fn rejected_late_death_drop(game: &mut Game, iterations: usize) {
  for _ in 0..iterations {
    let result = game.step(Command::AttackMelee(Direction::East));
    assert!(
      result.is_err(),
      "fixed late death-drop command must be rejected"
    );
    let _ = black_box(result);
  }
}

fn operations_per_second(elapsed_ns: u128, iterations: usize) -> u128 {
  (iterations as u128 * 1_000_000_000)
    .checked_div(elapsed_ns)
    .unwrap_or_default()
}

fn emit_measurement(
  case: &str,
  sample: usize,
  is_median: bool,
  config: BenchConfig,
  measurement: Measurement,
) {
  let ns_per_operation = measurement.elapsed_ns / config.iterations as u128;
  println!(
    "{{\"schema_version\":{SCHEMA_VERSION},\"benchmark\":\"{BENCHMARK_NAME}\",\"case\":\"{case}\",\"ownership\":\"core.rollback\",\"seed\":{BENCH_SEED},\"fixture_width\":{FIXTURE_WIDTH},\"fixture_height\":{FIXTURE_HEIGHT},\"timing_allocator_counting\":false,\"allocation_measurement\":\"separate_pass\",\"sample\":{sample},\"median\":{is_median},\"iterations\":{},\"warmup\":{},\"elapsed_ns\":{},\"ns_per_operation\":{ns_per_operation},\"operations_per_second\":{},\"allocations\":{},\"deallocations\":{},\"allocated_bytes\":{},\"deallocated_bytes\":{}}}",
    config.iterations,
    config.warmup,
    measurement.elapsed_ns,
    operations_per_second(measurement.elapsed_ns, config.iterations),
    measurement.allocations,
    measurement.deallocations,
    measurement.allocated_bytes,
    measurement.deallocated_bytes,
  );
}

fn median(measurements: &[Measurement]) -> Measurement {
  assert!(
    !measurements.is_empty(),
    "benchmark requires at least one sample"
  );
  let mut ordered = measurements.to_vec();
  ordered.sort_unstable_by_key(|measurement| measurement.elapsed_ns);
  ordered[ordered.len() / 2]
}

fn run_case(
  case: &str,
  config: BenchConfig,
  create_game: fn() -> Game,
  workload: fn(&mut Game, usize),
) {
  let mut measurements = Vec::with_capacity(config.samples);
  for sample in 0..config.samples {
    let measurement = measure(create_game, config, workload);
    emit_measurement(case, sample, false, config, measurement);
    measurements.push(measurement);
  }
  emit_measurement(case, 0, true, config, median(&measurements));
}

fn wait_game() -> Game {
  Game::new_arena(BENCH_SEED, FIXTURE_WIDTH, FIXTURE_HEIGHT).expect("fixed arena")
}

fn move_game() -> Game {
  Game::new(
    BENCH_SEED,
    FIXTURE_WIDTH,
    FIXTURE_HEIGHT,
    Position::new(2, 1),
  )
  .expect("fixed arena")
}

fn blocked_move_game() -> Game {
  Game::new(
    BENCH_SEED,
    FIXTURE_WIDTH,
    FIXTURE_HEIGHT,
    Position::new(1, 1),
  )
  .expect("fixed arena")
}

fn out_of_bounds_ranged_game() -> Game {
  Game::new(
    BENCH_SEED,
    FIXTURE_WIDTH,
    FIXTURE_HEIGHT,
    Position::new(1, 1),
  )
  .expect("fixed arena")
}

fn late_death_drop_game() -> Game {
  let mut game = Game::new_arena(BENCH_SEED, FIXTURE_WIDTH, FIXTURE_HEIGHT).expect("fixed arena");
  let target_position = Position::new(11, 7);
  let target_id = game
    .world_mut()
    .spawn_monster(target_position, "Dropper", 1, 0, (1, 1))
    .expect("fixed target");
  game
    .world_mut()
    .get_actor_mut(target_id)
    .expect("target actor")
    .set_death_drop(Some(ItemSpawnKind::SmallMedPack));
  game
    .world_mut()
    .map_mut()
    .set_tile(target_position, Tile::Wall);
  game
}

fn json_string(value: &str) -> String {
  let mut escaped = String::with_capacity(value.len());
  for character in value.chars() {
    match character {
      '\\' => escaped.push_str("\\\\"),
      '"' => escaped.push_str("\\\""),
      '\n' => escaped.push_str("\\n"),
      '\r' => escaped.push_str("\\r"),
      '\t' => escaped.push_str("\\t"),
      _ => escaped.push(character),
    }
  }
  escaped
}

fn main() {
  let config = BenchConfig::parse();
  let revision = std::env::var("DRL_BENCH_REVISION").unwrap_or_else(|_| "unknown".to_string());
  let rust_version =
    std::env::var("DRL_BENCH_RUST_VERSION").unwrap_or_else(|_| "unknown".to_string());
  let timestamp_unix = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_secs();
  println!(
    "{{\"schema_version\":{SCHEMA_VERSION},\"benchmark\":\"{BENCHMARK_NAME}\",\"kind\":\"metadata\",\"revision\":\"{}\",\"timestamp_unix\":{timestamp_unix},\"rust_version\":\"{}\",\"arch\":\"{}\",\"os\":\"{}\",\"profile\":\"bench\",\"seed\":{BENCH_SEED},\"fixture_width\":{FIXTURE_WIDTH},\"fixture_height\":{FIXTURE_HEIGHT},\"timing_allocator_counting\":false,\"allocation_measurement\":\"separate_pass\",\"ownership\":\"core.rollback\"}}",
    json_string(&revision),
    json_string(&rust_version),
    std::env::consts::ARCH,
    std::env::consts::OS,
  );

  run_case("core.accepted.wait", config, wait_game, accepted_wait);
  run_case("core.accepted.move", config, move_game, accepted_move);
  run_case(
    "core.rejected.blocked_move",
    config,
    blocked_move_game,
    rejected_blocked_move,
  );
  run_case(
    "core.rejected.out_of_bounds_ranged",
    config,
    out_of_bounds_ranged_game,
    rejected_out_of_bounds_ranged,
  );
  run_case(
    "core.rejected.late_death_drop",
    config,
    late_death_drop_game,
    rejected_late_death_drop,
  );
}
