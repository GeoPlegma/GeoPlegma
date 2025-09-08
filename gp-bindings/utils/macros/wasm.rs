// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

// Macros that will be used later for wasm
// keeping both macro separate is often more maintainable
// and avoids implicit type guessing

// This one is for int, floats, etc.
#[macro_export]
macro_rules! wasm_fields_copy {
    ($struct_name:ident, $( ($getter:ident, $setter:ident, $field_ident:ident, $field_str:literal, $ty:ty) ),* $(,)?) => {
        #[wasm_bindgen]
        impl $struct_name {
            $(
                #[wasm_bindgen(getter)]
                pub fn $getter(&self) -> $ty {
                    self.$field_ident
                }

                #[wasm_bindgen(setter = $field_str)]
                pub fn $setter(&mut self, val: $ty) {
                    self.$field_ident = val;
                }
            )*
        }
    };
}

// This one is for more complex types
#[macro_export]
macro_rules! wasm_fields_clone {
    ($struct_name:ident, $( ($getter:ident, $setter:ident, $field_ident:ident, $field_str:literal, $ty:ty) ),* $(,)?) => {
        #[wasm_bindgen]
        impl $struct_name {
            $(
                #[wasm_bindgen(getter = $field_str)]
                pub fn $getter(&self) -> $ty {
                    self.$field_ident.clone()
                }

                #[wasm_bindgen(setter = $field_str)]
                pub fn $setter(&mut self, val: $ty) {
                    self.$field_ident = val;
                }
            )*
        }
    };
}
