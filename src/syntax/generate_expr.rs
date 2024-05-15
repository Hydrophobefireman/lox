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

#[macro_export]
macro_rules! gen_native_func {
    ($fn_name:ident, $int:ident, $body:expr  $(, $arg:ident)*  ) => {

    #[derive(Debug)]
    struct $fn_name;

    impl LoxCallable for $fn_name {
        fn arity(&self)->usize {
            let count = 0$( +{let $arg= 1;$arg} )*;
            count
        }

        fn name(&self) ->String {
            String::from(stringify!(fn_name))
        }
        fn kind(&self) -> LoxCallableType {
            LoxCallableType::NativeFunction
        }

        fn call(&mut self, interpreter:Interpreter,_args:Vec<LoxType>) -> RuntimeResult<(LoxType, Interpreter)> {
            #[allow(unused_mut)]
            let  $int = interpreter;
            let mut _idx = 0;
            $( let $arg= &_args[_idx];_idx+=1; )*
            Ok(($body,$int))
        }
    }
    impl From<$fn_name> for LoxType {
        fn from(value: $fn_name) -> Self {
            LoxType::Callable(ref_cell(value))
        }
    }
    };
}

#[macro_export]
macro_rules! native_args {
    () => {};
}
