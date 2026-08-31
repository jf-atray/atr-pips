#[macro_export]
macro_rules! addition {
    (
        $(#[$meta:meta])*
        $vis:vis struct $module:ident : $world:ident {
            tables: {
                $($tfield:ident : Class<$ftype:ty $(, $ktype:ty)?> = $texpr:expr),+ $(,)?
            },
            solvers: { $($sfield:ident : $sty:ty = $sexpr:expr),* $(,)? },
            scripts: { $($cfield:ident : $cty:ty = $cexpr:expr),* $(,)? },
            signals: { $($gfield:ident : $gty:ty = $gexpr:expr),* $(,)? },
        }
    ) => {
        $(#[$meta])*
        $vis struct $world {}

        mod $module {
            use super::*;

            $crate::partition! {
                pub struct Tables as View for $world {
                    $(pub $tfield : Class<$ftype $(, $ktype)?>,)+
                }
            }

            impl $crate::addition::Tables for Tables {
                fn for_each_table(
                    &self,
                    f: &mut dyn FnMut(&'static str, &dyn $crate::ecs::Table),
                ) {
                    $(
                        f(stringify!($tfield), &self.$tfield as &dyn $crate::ecs::Table);
                    )*
                }
            }

            #[derive(Debug)]
            pub struct Solvers {
                $(pub $sfield: $sty),*
            }

            impl $crate::addition::Solvers for Solvers {
                #[allow(unused_variables)]
                fn update(
                    &mut self,
                    dt: f32,
                    pips: &mut $crate::addition::Pips,
                    scripts: &mut $crate::addition::ScriptsMap,
                    signals: &mut $crate::addition::SignalsMap,
                    input: &mut $crate::input::Input,
                    asset_registry: &std::collections::HashMap<String, $crate::assets::SpriteEntry>,
                ) {
                    $(self.$sfield.update(dt, pips, scripts, signals, input, asset_registry);)*
                }

                fn for_each_solver(
                    &mut self,
                    f: &mut dyn FnMut(&'static str, &mut dyn $crate::addition::Solver),
                ) {
                    $(
                        f(stringify!($sfield), &mut self.$sfield as &mut dyn $crate::addition::Solver);
                    )*
                }
            }

            #[derive(Debug)]
            pub struct Scripts {
                $(pub $cfield: $cty),*
            }
            impl $crate::addition::Scripts for Scripts {}

            #[derive(Debug)]
            pub struct Signals {
                $(pub $gfield: $gty),*
            }
            impl $crate::addition::Signals for Signals {}
        }

        impl $crate::addition::Addition for $world {
            type Tables = $module::Tables;
            type Solvers = $module::Solvers;
            type Scripts = $module::Scripts;
            type Signals = $module::Signals;
            type View = $module::View;

            fn make_tables() -> Self::Tables {
                $module::Tables { $($tfield: $texpr),+ }
            }
            fn make_solvers() -> Self::Solvers {
                $module::Solvers { $($sfield: $sexpr),* }
            }
            fn make_scripts() -> Self::Scripts {
                $module::Scripts { $($cfield: $cexpr),* }
            }
            fn make_signals() -> Self::Signals {
                $module::Signals { $($gfield: $gexpr),* }
            }
        }
    };
}
