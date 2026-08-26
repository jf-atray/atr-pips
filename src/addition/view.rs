use super::addition::Addition;
use super::traits::{Tables, Solvers, Scripts, Signals};

#[derive(Debug)]
pub struct ViewMut<'domain, T, K, M, N>
where
    T: Tables,
    K: Solvers,
    M: Scripts,
    N: Signals,
{
    pub tables: &'domain mut T,
    pub solvers: &'domain mut K,
    pub scripts: &'domain mut M,
    pub signals: &'domain mut N,
}

impl<'domain, T, K, M, N> ViewMut<'domain, T, K, M, N>
where
    T: Tables,
    K: Solvers,
    M: Scripts,
    N: Signals,
{
    pub fn new(
        tables: &'domain mut T,
        solvers: &'domain mut K,
        scripts: &'domain mut M,
        signals: &'domain mut N,
    ) -> Self {
        Self {
            tables,
            solvers,
            scripts,
            signals,
        }
    }
}

pub type AsViewMut<'a, A> = ViewMut<
    'a,
    <A as Addition>::Tables,
    <A as Addition>::Solvers,
    <A as Addition>::Scripts,
    <A as Addition>::Signals,
>;
