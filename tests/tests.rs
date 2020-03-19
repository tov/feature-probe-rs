extern crate feature_probe;

use feature_probe::*;

#[macro_use]
mod helpers;

fn build_probe() -> Probe { Probe::new() }

probe_tests! {

    good_types          { mod
        u32             {   probe_type("u32") }
        i16             {   probe_type("i16") }
        string          {   probe_type("String") }
        qual_string     {   probe_type("std::string::String") }
        static_slice    {   probe_type("&'static str") }
        str_slice       {   probe_type("&str") }
        str_unsized     {   probe_type("str") }
        vec_opt_bool    {   probe_type("Vec<Option<bool>>") }
    }

    bad_types           { mod
        q32             { ! probe_type("q32") }
        i2048           { ! probe_type("i2048") }
        strang          { ! probe_type("Strang") }
    }

    rust_expressions    { mod
        true_exp        {   probe_expression("true") }
        add_int_exp     {   probe_expression("5 + 6") }
        range_exp       {   probe_expression("0..10") }
        vec_new_amb     { ! probe_expression("Vec::new()") }
        vec_new_unamb   {   probe_typed_expression("Vec::new()", "Vec<u16>") }
    }

    perl_expressions    { mod
        weird1          { ! probe_expression("$_") }
        weird2          { ! probe_expression("/a.*b/g") }
        weird3          { ! probe_expression("$Package::Hash{ 'the key'}") }
    }
    
}

