#[macro_export]
macro_rules! generate_expr_struct {
    ($($variant:ident $(, $field:ident: $ty:ty)*);*) => {

        pub trait Visitor<R> {
            $(
                #[allow(non_snake_case)]
                fn $variant(&self, e: &$variant) -> R;
            )*
        }

        $(pub struct $variant {
             $(pub $field: $ty),*
        }

        impl $variant {
            #[allow(dead_code)]
            pub fn new($($field: $ty),*) -> Self {
                Self {
                    $($field),*
                }
            }
            #[allow(dead_code)]
            pub fn accept<T:Visitor<R>,R>(&self,x:&T)->R{
                x.$variant(self)
            }
        })*

        #[allow(dead_code)]
        pub enum Expr {

            $(
                $variant($variant),
            )*
        }
        impl Expr {
            #[allow(dead_code)]
            pub fn accept<T: Visitor<R>, R>(&self, x: &T) -> R {
                match self {
                    $(Expr::$variant(v) => v.accept(x),)*
                }
            }
        }
    };
}
