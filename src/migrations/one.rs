use crate::{Storage, MIGRATION};
use linkme::distributed_slice;
use migrations_mre_linkme::InnerMigration;

fn init(s: &mut Storage) {
    s.insert(11, "One initialized.");
}

fn update(s: &mut Storage) {
    s.insert(1, "One");
}

#[distributed_slice(MIGRATION)]
static ONE: InnerMigration<Storage> =
    InnerMigration::new_initialize_update(&init, &update, "One", "The one migration");
