pub mod impls;

#[macro_export]
macro_rules! query {
    ([& $x:expr $(,)?], $callback:expr) => {
        $crate::query::impls::query_ref(&$x)
            .for_each(|col| col.iter().for_each(|a| $crate::query::impls::call1($callback, a)));
    };

    ([&mut $x:expr $(,)?], $callback:expr) => {
        $crate::query::impls::query_mut(&mut $x)
            .for_each(|col| col.iter_mut().for_each(|a| $crate::query::impls::call1($callback, a)));
    };

    ([$ak:expr; &mut $x:expr, $bk:expr; &mut $y:expr $(,)?], $callback:expr) => {
        $crate::query::impls::query_mut_mut(&mut $x, &$ak, &mut $y, &$bk)
            .for_each(|(cols_a, cols_b)| {
                std::iter::zip(cols_a, cols_b)
                    .for_each(|(a, b)| $crate::query::impls::call2($callback, a, b));
            });
    };

    ([&mut $x:expr, &mut $y:expr $(,)?], $callback:expr) => {
        $crate::query!([(); &mut $x, (); &mut $y], $callback);
    };

    ([$ak:expr; &mut $x:expr, $bk:expr; & $y:expr $(,)?], $callback:expr) => {
        $crate::query::impls::query_mut_ref(&mut $x, &$ak, &$y, &$bk)
            .for_each(|(cols_a, cols_b)| {
                std::iter::zip(cols_a, cols_b)
                    .for_each(|(a, b)| $crate::query::impls::call2($callback, a, b));
            });
    };

    ([&mut $x:expr, & $y:expr $(,)?], $callback:expr) => {
        $crate::query!([(); &mut $x, (); & $y], $callback);
    };

    ([$ak:expr; & $x:expr, $bk:expr; &mut $y:expr $(,)?], $callback:expr) => {
        $crate::query::impls::query_mut_ref(&mut $y, &$bk, &$x, &$ak)
            .for_each(|(cols_a, cols_b)| {
                std::iter::zip(cols_a, cols_b)
                    .for_each(|(a, b)| $crate::query::impls::call2($callback, b, a));
            });
    };

    ([& $x:expr, &mut $y:expr $(,)?], $callback:expr) => {
        $crate::query!([(); & $x, (); &mut $y], $callback);
    };

    ([$ak:expr; & $x:expr, $bk:expr; & $y:expr $(,)?], $callback:expr) => {
        $crate::query::impls::query_ref_ref(&$x, &$ak, &$y, &$bk)
            .for_each(|(cols_a, cols_b)| {
                std::iter::zip(cols_a, cols_b)
                    .for_each(|(a, b)| $crate::query::impls::call2($callback, a, b));
            });
    };

    ([& $x:expr, & $y:expr $(,)?], $callback:expr) => {
        $crate::query!([(); & $x, (); & $y], $callback);
    };

    ([&mut $x:expr, &mut $y:expr, &mut $z:expr $(,)?], $callback:expr) => {
        $crate::query!([(); &mut $x, (); &mut $y, (); &mut $z], $callback);
    };

    ([$ak:expr; &mut $x:expr, $bk:expr; &mut $y:expr, $ck:expr; &mut $z:expr $(,)?], $callback:expr) => {
        $crate::query::impls::query_mut_mut_mut(&mut $x, &$ak, &mut $y, &$bk, &mut $z, &$ck)
            .for_each(|(cols_a, cols_b, cols_c)| {
                std::iter::zip(std::iter::zip(cols_a, cols_b), cols_c)
                    .for_each(|((a, b), c)| $crate::query::impls::call3($callback, a, b, c));
            });
    };
}
