use std::any::{Any, TypeId};

use crate::tables::ClassId;

pub trait View: Any {
    fn width(&self) -> usize;
    fn matches(&self, class_id: ClassId, into: &dyn Any) -> bool;
    fn commit(&mut self, class_id: ClassId, into: &mut dyn Any) -> Option<usize>;
    fn addition_id(&self) -> TypeId;
    fn as_any_ref(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait Addition: Any {
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
        $vis struct $addition {
            $($fvis $fname: $crate::tables::class::Class<$ftype $(, $ktype)?>, )+
        }

        #[derive(Default)]
        $vis struct $view {
            $($fvis $fname: Option<$ftype>, )+
        }

        impl $crate::tables::partition::View for $view {
            fn width(&self) -> usize {
                0usize $(+ self.$fname.is_some() as usize)+
            }

            fn matches(&self, class_id: $crate::tables::ClassId, into: &dyn ::std::any::Any) -> bool {
                let into = into.downcast_ref::<$addition>().unwrap();
                true $(&& self.$fname.is_some() == into.$fname.get_col(class_id).is_some())+
            }

            fn commit(&mut self, class_id: $crate::tables::ClassId, into: &mut dyn ::std::any::Any) -> Option<usize> {
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

        impl $crate::tables::partition::Addition for $addition {
            fn view_default(&self) -> Box<dyn $crate::tables::partition::View> {
                Box::new($view::default())
            }

            fn destroy(&mut self, class_id: $crate::tables::ClassId, row_idx: usize) {
                $(
                    if let Some(col) = self.$fname.get_col_mut(class_id) {
                        col.swap_remove(row_idx);
                    }
                )+
            }

            fn clear(&mut self) {
                $( self.$fname.clear(); )+
            }
        }
    };
}
