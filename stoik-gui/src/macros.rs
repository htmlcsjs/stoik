macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        name.strip_suffix("::f").unwrap()
    }};
}

macro_rules! trace {
    ($ui: expr) => {
        egui::trace!($ui, $crate::macros::function!())
    };
}

pub(crate) use function;
pub(crate) use trace;
