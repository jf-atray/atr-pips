use std::any::{Any, TypeId};

use crate::ecs::ClassId;

pub trait View: Any {
    fn width(&self) -> usize;
    fn matches(&self, class_id: ClassId, into: &dyn Partition) -> bool;
    fn commit(&mut self, class_id: ClassId, into: &mut dyn Partition) -> Option<usize>;
    fn addition_id(&self) -> TypeId;
    fn as_any_ref(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait Partition: Any + std::fmt::Debug {
    fn view_default(&self) -> Box<dyn View>;
    fn destroy(&mut self, class_id: ClassId, row_idx: usize);
    fn clear(&mut self);
}

#[macro_export]
macro_rules! partition {
    (
        $vis:vis struct $addition:ident as $view:ident {
            $($fvis:vis $fname:ident : Class<$ftype:ty $(, $ktype:ty)?>,)+
        }
    ) => {
        #[derive(Debug)]
        $vis struct $addition {
            $($fvis $fname: $crate::ecs::class::Class<$ftype $(, $ktype)?>, )+
        }

        #[derive(Default)]
        $vis struct $view {
            $($fvis $fname: Option<$ftype>, )+
        }

        impl $addition {
            pub fn new() -> Self {
                Self {
                    $($fname: $crate::ecs::class::Class::new(
                        $crate::ecs::class_strategy::GrowthStrategy::quart_kib::<$ftype>(),
                    ),)+
                }
            }

            pub fn with_opinions<F>(f: F) -> Self
            where
                F: FnOnce(&mut Self),
            {
                let mut addition = Self::new();
                f(&mut addition);
                addition
            }
        }

        impl $view {
            #[allow(clippy::too_many_arguments)]
            pub fn with(&mut self, $($fname:$ftype, )+) -> &mut Self {
                $(self.$fname = Some($fname); )+
                self
            }
        }

        impl $crate::ecs::partition::View for $view {

            fn width(&self) -> usize {
                0usize $(+ usize::from(self.$fname.is_some()))+
            }

            fn matches(&self, class_id: $crate::ecs::ClassId, into: &dyn $crate::ecs::partition::Addition) -> bool {
                let into: &dyn ::std::any::Any = into;
                let into = into.downcast_ref::<$addition>().unwrap();
                true $(&& self.$fname.is_some() == into.$fname.data.get(class_id).is_some())+
            }

            fn commit(&mut self, class_id: $crate::ecs::ClassId, into: &mut dyn $crate::ecs::partition::Addition) -> Option<usize> {
                let into: &mut dyn ::std::any::Any = into;
                let into = into.downcast_mut::<$addition>().unwrap();
                let mut row = None;
                $(
                    if let Some(v) = self.$fname.take() {
                        let col = into.$fname.get_col_or_insert(class_id);
                        if row.is_none() {
                            row = Some(col.len());
                        }
                        col.push(v);
                    }
                )+
                row
            }

            fn addition_id(&self) -> ::std::any::TypeId {
                ::std::any::TypeId::of::<$addition>()
            }

            fn as_any_ref(&self) -> &dyn ::std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any {
                self
            }
        }

        impl $crate::ecs::partition::Addition for $addition {
            fn view_default(&self) -> Box<dyn $crate::ecs::partition::View> {
                Box::new($view::default())
            }

            fn destroy(&mut self, class_id: $crate::ecs::ClassId, row_idx: usize) {
                $(
                    if let Some(col) = self.$fname.data.get_mut(class_id) {
                        col.swap_remove(row_idx);
                    }
                )+
            }

            fn clear(&mut self) {
                $( self.$fname.data.clear(); )+
            }
        }
    };
}
