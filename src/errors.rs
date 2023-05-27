macro_rules! err_struct {
    ($name:ident,$err:ident) => {
        #[derive(Debug)]
        pub struct $name {
            pub message: String,
            pub line: usize,
        }
        impl $name {
            pub fn new(message: &str, line: usize) -> Self {
                Self {
                    message: message.into(),
                    line,
                }
            }
        }
        pub type $err<T> = Result<T, $name>;
    };
}
err_struct!(RuntimeError, RuntimeResult);

err_struct!(ScanError, ScanResult);

err_struct!(ParseError, ParseResult);
