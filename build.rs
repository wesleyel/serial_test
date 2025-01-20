use std::process::Command;

fn main() {
    // 获取当前的 Git 标签
    let tag_output = Command::new("git")
        .args(&["describe", "--tags", "--exact-match"])
        .output()
        .expect("Failed to execute git command");

    // 获取当前的 Git 修订版本
    let rev_output = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .expect("Failed to execute git command");

    // 获取当前的 Git 分支
    let branch_output = Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .expect("Failed to execute git command");

    let tag = String::from_utf8_lossy(&tag_output.stdout)
        .trim()
        .to_string();
    let rev = String::from_utf8_lossy(&rev_output.stdout)
        .trim()
        .to_string();
    let branch = String::from_utf8_lossy(&branch_output.stdout)
        .trim()
        .to_string();

    // 判断是否有标签
    let version = if !tag.is_empty() && tag == branch {
        env!("CARGO_PKG_VERSION").to_string()
    } else {
        format!("{}-{}", env!("CARGO_PKG_VERSION"), rev)
    };

    // 将版本信息传递给编译器
    println!("cargo:rustc-env=GIT_VERSION={}", version);
}
