use super::traits::{Tables, Solvers, Scripts, Signals};

pub(super) trait Addition {
    type Tables: Tables;
    type Solvers: Solvers;
    type Scripts: Scripts;
    type Signals: Signals;
    fn make_tables() -> Self::Tables;
    fn make_solvers() -> Self::Solvers;
    fn make_scripts() -> Self::Scripts;
    fn make_signals() -> Self::Signals;
}
