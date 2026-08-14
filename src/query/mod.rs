pub mod impls;

#[macro_export]
macro_rules! query {
    
    ([$ak:expr; &mut $x:expr, $bk:expr; &mut $y:expr $(,)?], |$a:ident, $b:ident| $body:block) => {
        $crate::query::impls::query_mut_mut(&mut $x, &$ak, &mut $y, &$bk)
            .for_each(|(__cols_a, __cols_b)| {
                for (__a, __b) in std::iter::zip(__cols_a, __cols_b) {
                    let $a = __a;
                    let $b = __b;
                    $body
                }
            });
    };
    ([&mut $x:expr, &mut $y:expr $(,)?], |$a:ident, $b:ident| $body:block) => {
        $crate::query::impls::query_mut_mut(&mut $x, &(), &mut $y, &())
            .for_each(|(__cols_a, __cols_b)| {
                for (__a, __b) in std::iter::zip(__cols_a, __cols_b) {
                    let $a = __a;
                    let $b = __b;
                    $body
                }
            });
    };


    ([$ak:expr; & $x:expr, $bk:expr; & $y:expr $(,)?], |$a:ident, $b:ident| $body:block) => {
        $crate::query::impls::query_ref_ref(&$x, &$ak, &$y, &$bk)
            .for_each(|(__cols_a, __cols_b)| {
                for (__a, __b) in std::iter::zip(__cols_a, __cols_b) {
                    let $a = __a;
                    let $b = __b;
                    $body
                }
            });
    };
    ([& $x:expr, & $y:expr $(,)?], |$a:ident, $b:ident| $body:block) => {
        $crate::query::impls::query_ref_ref(&$x, &(), &$y, &())
            .for_each(|(__cols_a, __cols_b)| {
                for (__a, __b) in std::iter::zip(__cols_a, __cols_b) {
                    let $a = __a;
                    let $b = __b;
                    $body
                }
            });
    };
}
