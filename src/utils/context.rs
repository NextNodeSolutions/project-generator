use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static VARIABLES: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
    static DEBUG_MODE: RefCell<bool> = const { RefCell::new(false) };
}

pub fn set_variables(vars: HashMap<String, String>) {
    VARIABLES.with(|v| *v.borrow_mut() = vars);
}

pub fn get_variable(key: &str) -> Option<String> {
    VARIABLES.with(|v| v.borrow().get(key).cloned())
}

pub fn set_debug_mode(debug: bool) {
    DEBUG_MODE.with(|d| *d.borrow_mut() = debug);
}

pub fn is_debug_mode() -> bool {
    DEBUG_MODE.with(|d| *d.borrow())
}

pub fn debug_print(message: &str) {
    if is_debug_mode() {
        println!("[DEBUG] {}", message);
    }
}
