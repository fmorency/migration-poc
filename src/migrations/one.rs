use crate::{Storage, MIG};
use linkme::distributed_slice;
use migrations_mre_linkme::InnerMig;

fn init(s: &mut Storage) {
    s.insert(11, "One initialized.");
}

fn update(s: &mut Storage) {
    s.insert(1, "One");
}

fn desc() -> (&'static str, &'static str) {
    ("One", "The one migration")
}

#[distributed_slice(MIG)]
static ONE: InnerMig<Storage> = InnerMig::new_init_update(&init, &update, desc);
