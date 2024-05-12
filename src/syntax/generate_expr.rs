#[macro_export]
macro_rules! gen_struct {
    ($st_name:ident, $($variant:ident $(, $field:ident: $ty:ty)*);*) => {

        // pub trait Visitor<Result> : Sized {
        //     $(
        //         #[allow(non_snake_case)]
        //         fn $variant( self, e: $variant) -> Result;
        //     )*
        // }

        $(#[derive(Debug, Clone)]
        pub struct $variant {
             $(pub $field: $ty),*
        }

        impl $variant {
            #[allow(dead_code)]
            pub fn new($($field: $ty),*) -> Self {
                Self { $($field),* }
            }

            // #[allow(dead_code)]
            // pub fn accept<T:Visitor<Res>,Res>(self,x:T)->Res{
            //     x.$variant(self)
            // }
        })*

        $(

            impl From<$variant> for $st_name {

                fn from(value:$variant) ->Self {
                    Self::$variant(value)
                }
            }

        )*

        #[allow(dead_code)]
        #[derive(Debug,Clone)]
        pub enum $st_name {

            $(
                $variant($variant),
            )*
        }
        // impl $st_name {
        //     #[allow(dead_code)]
        //     pub fn accept<T: Visitor<Res>, Res>(self, x: T) -> Res {
        //         let ax = self;
        //         match ax {
        //             $($st_name::$variant(v) => {
        //                  v.accept(x)
        //             },)*
        //         }
        //     }
        // }



    };
}
