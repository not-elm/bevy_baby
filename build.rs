use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    if target.contains("windows") {
        // on windows we will set our game icon as icon for the executable
        let _ = embed_resource::compile("build/windows/icon.rc", embed_resource::NONE);
    }
}
