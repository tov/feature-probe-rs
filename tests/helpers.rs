macro_rules! new_probe {
    {
        $( #[ $($prop:ident=$val:expr),* $(,)? ] )?
        $(, $qual:ident)?
    } => {
        {
            #[allow(unused_mut)]
            let mut probe = $($qual::)? build_probe();
            $($( probe.$prop($val); )*)?
            probe
        }
    }
}

macro_rules! probe_test {
    {
        $(#$prop:tt)?
        $name:ident { $meth:ident($($arg:tt)*) }
    } => {
        #[test]
        fn $name() {
            let o = new_probe!($(#$prop)?);
            assert!( o.$meth($($arg)*) );
        }
    };

    {
        $(#$prop:tt)?
        $name:ident { ! $meth:ident($($arg:tt)*) }
    } => {
        #[test]
        fn $name() {
            let o = new_probe!($(#$prop)?);
            assert!( ! o.$meth($($arg)*) );
        }
    };

    {
        $(#$prop:tt)?
        $name:ident { mod $($inner:tt)* }
    } => {
        mod $name {
            fn build_probe() -> feature_probe::Probe {
                new_probe!($(#$prop)?, super)
            }

            probe_tests! { $($inner)* }
        }
    }
}

macro_rules! probe_tests {
    {
        $(
        $(#$prop:tt)?
        $name:ident { $($body:tt)* }
        )*
    } =>
    {
        $(
        probe_test! {
            $(#$prop)?
            $name { $($body)* }
        }
        )*
    }
}
