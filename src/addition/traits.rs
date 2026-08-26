use downcast_rs::{Downcast, impl_downcast};

pub(super) trait Tables: Downcast {}
pub(super) trait Solvers: Downcast {}
pub(super) trait Solver: Downcast {}
impl<T: Solver> Solvers for T {}

pub(super) trait Scripts: Downcast {}
pub(super) trait Signals: Downcast {}

impl_downcast!(Tables);
impl_downcast!(Solvers);
impl_downcast!(Scripts);
impl_downcast!(Signals);
