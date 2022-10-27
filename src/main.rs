use linkme::distributed_slice;
use migrations_mre_linkme::{load_enable_all_regular_migrations, load_migrations, InnerMigration};
use minicbor::{Decode, Encode};
use std::collections::BTreeMap;
use std::thread::sleep;
use std::time::Duration;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod migrations;

const MIGRATION_CONFIG: &str = r#"
[
  {
    "type": "One",
    "block_height": 2,
    "issue": "https://github.com/liftedinit/many-framework/issues/190"
  },
  {
    "type": "Two",
    "block_height": 5
  },
  {
    "type": "One",
    "block_height": 3
  },
  {
    "type": "Three",
    "block_height": 7
  },
  {
    "type": "Four",
    "block_height": 7
  }
]
"#;

pub type Storage = BTreeMap<u8, &'static str>;

// This is the global migration registry
// Doesn't contain any metadata
#[distributed_slice]
pub static MIGRATION: [InnerMigration<'static, Storage>] = [..];

#[derive(Encode, Decode)]
struct Dummy(#[n(0)] u64);

fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // `v` is the active migration list
    let mut v = load_migrations(&MIGRATION, MIGRATION_CONFIG).unwrap();
    // let mut v = load_enable_all_regular_migrations(&MIGRATION);

    // Direct access to the migration
    println!("Migration Three is enabled? {}", v["Three"].is_enabled());

    println!("Disabling Three");
    v.entry("Three").and_modify(|x| x.disable());
    println!("Migration Three is enabled? {}", v["Three"].is_enabled());
    //
    for i in v.values() {
        // for i in v {
        println!("NAME: {} - DESC: {}", i.name(), i.description());
        println!("BLOCK HEIGHT: {}", i.metadata().block_height);
        println!("ISSUE: {:?}", i.metadata().issue);
        println!("STATUS: {}", i.status());
    }

    println!("Displaying migration registry info...");
    for i in MIGRATION {
        let name = i.name();
        let description = i.description();
        println!("{name} - {description}");
    }
    println!();

    let mut storage = Storage::new();
    let dummy = Dummy(0);

    for c in 0..10 {
        println!("Counter: {c}");

        println!("Initializing migrations...");
        for i in v.values() {
            i.initialize(&mut storage, c);
            println!("{storage:?}");
        }
        println!("Performing update...");
        for i in v.values() {
            i.update(&mut storage, c);
            println!("{storage:?}");
        }

        println!("Performing hotfix...");
        for i in v.values() {
            let r = i.hotfix(&minicbor::to_vec(&dummy).unwrap(), c);
            if let Some(a) = r {
                let d: Dummy = minicbor::decode(&a).unwrap();
                println!("Hotfix result: {}", d.0);
            }
        }

        println!();
        sleep(Duration::from_secs(1));
    }
}
