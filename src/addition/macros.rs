#[macro_export]
macro_rules! addition {
    (
        $(#[$meta:meta])*
        $world:ident {
            tables: $tables:ident { $($tfield:ident : $tty:ty = $texpr:expr),+ $(,)? },
            solvers: $solvers:ident { $($sfield:ident : $sty:ty = $sexpr:expr),+ $(,)? },
            scripts: $scripts:ident { $($cfield:ident : $cty:ty = $cexpr:expr),* $(,)? },
            signals: $signals:ident { $($gfield:ident : $gty:ty = $gexpr:expr),* $(,)? },
        }
    ) => {
        $(#[$meta])*
        struct $world {}

        $(#[$meta])*
        struct $tables {
            $($tfield: $tty),+
        }
        impl $crate::addition::Tables for $tables {}

        $(#[$meta])*
        struct $solvers {
            $($sfield: $sty),+
        }

        $(impl $crate::addition::Solver for $sty {})+

        impl $crate::addition::Solvers for $solvers {
            fn update(
                &mut self,
                dt: f32,
                tables: &mut $crate::addition::TypedMap<dyn $crate::addition::Tables>,
                scripts: &mut $crate::addition::TypedMap<dyn $crate::addition::Scripts>,
                signals: &mut $crate::addition::TypedMap<dyn $crate::addition::Signals>,
            ) {
                $(self.$sfield.update(dt, tables, scripts, signals);)+
            }
        }

        $(#[$meta])*
        struct $scripts {
            $($cfield: $cty),*
        }
        impl $crate::addition::Scripts for $scripts {}

        $(#[$meta])*
        struct $signals {
            $($gfield: $gty),*
        }
        impl $crate::addition::Signals for $signals {}

        impl $crate::addition::Addition for $world {
            type Tables = $tables;
            type Solvers = $solvers;
            type Scripts = $scripts;
            type Signals = $signals;

            fn make_tables() -> Self::Tables { $tables { $($tfield: $texpr),+ } }
            fn make_solvers() -> Self::Solvers { $solvers { $($sfield: $sexpr),+ } }
            fn make_scripts() -> Self::Scripts { $scripts { $($cfield: $cexpr),* } }
            fn make_signals() -> Self::Signals { $signals { $($gfield: $gexpr),* } }
        }
    };
}
