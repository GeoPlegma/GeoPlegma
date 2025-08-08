// keeping both macro separate is often more maintainable
// and avoids implicit type guessing
// #[macro_export]
// macro_rules! wasm_fields_copy {
//     (
//         $struct_name:ident {
//             $($field:ident : $type:ty),* $(,)?
//         }
//     ) => {
//         #[wasm_bindgen]
//         impl $struct_name {
//             $(
//                 #[wasm_bindgen(getter)]
//                 pub fn $field(&self) -> $type {
//                     self.$field
//                 }

//                 #[wasm_bindgen(setter = "$field")]
//                 pub fn set_$field(&mut self, val: $type) {
//                     self.$field = val;
//                 }
//             )*
//         }
//     };
// }

// #[macro_export]
// macro_rules! wasm_fields_clone {
//     (
//         $struct_name:ident {
//             $($field:ident : $type:ty),* $(,)?
//         }
//     ) => {
//         #[wasm_bindgen]
//         impl $struct_name {
//             $(
//                 #[wasm_bindgen(getter)]
//                 pub fn $field(&self) -> $type {
//                     self.$field
//                 }

//                 #[wasm_bindgen(setter = "$field")]
//                 pub fn set_$field(&mut self, val: $type) {
//                     self.$field = val;
//                 }
//             )*
//         }
//     };
// }

#[macro_export]
macro_rules! wasm_getters_setters {
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
