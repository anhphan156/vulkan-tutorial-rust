use std::process::Command;

fn main() {
    let shaders = ["triangle.vert", "triangle.frag"];

    for s in shaders.iter() {
        println!("cargo::rerun-if-changed=shaders/{}", s);

        Command::new("glslc")
            .args(&[
                format!("shaders/{}", s).as_str(),
                "-o",
                format!("shaders/spv/{}.spv", s).as_str(),
            ])
            .status()
            .unwrap();
    }
}
