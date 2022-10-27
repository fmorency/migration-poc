use crate::{Storage, MIGRATION};
use linkme::distributed_slice;
use migrations_mre_linkme::InnerMigration;

fn init(s: &mut Storage) {
    s.insert(22, "Two initialized.");
}

fn update(s: &mut Storage) {
    s.insert(2, "two");
}

#[distributed_slice(MIGRATION)]
static TWO: InnerMigration<Storage> =
    InnerMigration::new_initialize_update(&init, &update, "Two", "The sequel!");
