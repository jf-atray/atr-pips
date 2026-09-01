use std::any::Any;

use crate::ecs::ClassId;

pub trait IntoComponent<T, K> {
    fn into_component(self) -> (T, K);
}

impl<T> IntoComponent<T, ()> for T {
    fn into_component(self) -> (T, ()) {
        (self, ())
    }
}

impl<T, K> IntoComponent<T, K> for (T, K) {
    fn into_component(self) -> (T, K) {
        self
    }
}

pub trait View: Any + std::fmt::Debug {
    fn width(&self) -> usize;
    fn matches(&self, class_id: ClassId, into: &dyn Partition) -> bool;
    fn commit(&mut self, class_id: ClassId, into: &mut dyn Partition) -> Option<usize>;
    fn reset(&mut self);
    fn as_any_ref(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait Partition: Any + std::fmt::Debug {
    fn view_default(&self) -> Box<dyn View>;
    fn destroy(&mut self, class_id: ClassId, row_idx: usize);
    fn extract_into(&mut self, class_id: ClassId, row_idx: usize, into: &mut dyn View);
    fn clear(&mut self);
}

#[macro_export]
macro_rules! __class_k {
    ($ftype:ty) => { () };
    ($ftype:ty, $ktype:ty) => { $ktype };
}

#[macro_export]
macro_rules! partition {
    (
        $vis:vis struct $addition:ident as $view:ident for $world:ident {
            $($fvis:vis $fname:ident : Class<$ftype:ty $(, $ktype:ty)?>,)+
        }
    ) => {
        #[derive(Debug)]
        $vis struct $addition {
            $($fvis $fname: $crate::ecs::class::Class<$ftype, $crate::__class_k!($ftype $(, $ktype)?)>, )+
        }

        #[derive(Default, Debug)]
        $vis struct $view {
            $($fvis $fname: Option<($ftype, $crate::__class_k!($ftype $(, $ktype)?))>, )+
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
            pub fn with(
                &mut self,
                $($fname: impl $crate::ecs::partition::IntoComponent<$ftype, $crate::__class_k!($ftype $(, $ktype)?)>, )+
            ) -> &mut Self {
                $(self.$fname = Some($fname.into_component()); )+
                self
            }
        }

        impl $crate::ecs::partition::View for $view {

            fn width(&self) -> usize {
                0usize $(+ usize::from(self.$fname.is_some()))+
            }

            fn matches(&self, class_id: $crate::ecs::ClassId, into: &dyn $crate::ecs::partition::Partition) -> bool {
                let into: &dyn ::std::any::Any = into;
                let into = into.downcast_ref::<$addition>().unwrap();
                true $(&& match (&self.$fname, into.$fname.data.get(class_id)) {
                    (None, None) => true,
                    (Some((_, k)), Some(col)) => col.key == *k,
                    _ => false,
                })+
            }

            fn commit(&mut self, class_id: $crate::ecs::ClassId, into: &mut dyn $crate::ecs::partition::Partition) -> Option<usize> {
                let into: &mut dyn ::std::any::Any = into;
                let into = into.downcast_mut::<$addition>().unwrap();
                let mut row = None;
                $(
                    if let Some((v, k)) = self.$fname.take() {
                        let col = into.$fname.get_col_or_insert_with_key(class_id, k);
                        if row.is_none() {
                            row = Some(col.len());
                        }
                        col.push(v);
                    }
                )+
                row
            }

            fn as_any_ref(&self) -> &dyn ::std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn ::std::any::Any {
                self
            }

            fn reset(&mut self) {
                $(self.$fname = None; )+
            }
        }

        impl $crate::ecs::partition::Partition for $addition {
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

            fn extract_into(&mut self, class_id: $crate::ecs::ClassId, row_idx: usize, into: &mut dyn $crate::ecs::partition::View) {
                let into = into.as_any_mut().downcast_mut::<$view>().unwrap();
                $(
                    if let Some(col) = self.$fname.data.get_mut(class_id) {
                        if row_idx < col.len() {
                            let k = col.key;
                            let v = col.swap_remove(row_idx);
                            into.$fname = Some((v, k));
                        }
                    }
                )+
            }

            fn clear(&mut self) {
                $( self.$fname.data.clear(); )+
            }
        }
    };
}

impl crate::ecs::partition::View for () {
    fn width(&self) -> usize {
        0
    }

    fn matches(&self, _class_id: crate::ecs::ClassId, _into: &dyn crate::ecs::partition::Partition) -> bool {
        true
    }

    fn commit(&mut self, _class_id: crate::ecs::ClassId, _into: &mut dyn crate::ecs::partition::Partition) -> Option<usize> {
        None
    }

    fn as_any_ref(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn reset(&mut self) {}
}

impl crate::ecs::partition::Partition for () {
    fn view_default(&self) -> Box<dyn crate::ecs::partition::View> {
        Box::new(())
    }

    fn destroy(&mut self, _class_id: crate::ecs::ClassId, _row_idx: usize) {}

    fn extract_into(&mut self, _class_id: crate::ecs::ClassId, _row_idx: usize, _into: &mut dyn crate::ecs::partition::View) {}

    fn clear(&mut self) {}
}


