pub mod impls;

#[macro_export]
macro_rules! query {
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
}
