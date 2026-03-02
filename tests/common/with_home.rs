use std::{
    env,
    path::Path,
    sync::{Mutex, OnceLock},
};

pub fn with_home<T>(home: &Path, f: impl FnOnce() -> T) -> T {
    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let _guard = ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("env lock should not be poisoned");

    let previous_home = env::var_os("HOME");

    unsafe {
        env::set_var("HOME", home);
    }

    let result = f();

    match previous_home {
        Some(old) => unsafe {
            env::set_var("HOME", old);
        },
        None => unsafe {
            env::remove_var("HOME");
        },
    }

    result
}
