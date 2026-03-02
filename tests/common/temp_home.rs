use std::{
    env, fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn temp_home_dir(test_name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after UNIX_EPOCH")
        .as_nanos();

    let path = env::temp_dir().join(format!(
        "rustic-calc-{test_name}-{}-{nanos}",
        std::process::id()
    ));

    fs::create_dir_all(&path).expect("temp HOME should be creatable");
    path
}
