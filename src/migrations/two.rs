use crate::{Storage, MIG};
use linkme::distributed_slice;
use migrations_mre_linkme::InnerMig;

fn init(s: &mut Storage) {
    s.insert(22, "Two initialized.");
}

fn update(s: &mut Storage) {
    s.insert(2, "two");
}

fn desc() -> (&'static str, &'static str) {
    ("Two", "The second!")
}

#[distributed_slice(MIG)]
static TWO: InnerMig<Storage> = InnerMig::new_init_update(&init, &update, desc);
