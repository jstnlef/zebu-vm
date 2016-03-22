mod test_ir;
mod test_compiler;

#[macro_export]
macro_rules! init_logger {
    ($level : expr) => {
        match simple_logger::init_with_level($level) {
            Ok(_) => {},
            Err(_) => {}
        }
    }
}