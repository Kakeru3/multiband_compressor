use std::env;

fn main() -> nih_plug_xtask::Result<()> {
    let mut args = env::args().collect::<Vec<_>>();
    let mut target = None;

    if let Some(pos) = args.iter().position(|a| a == "--target") {
        if let Some(t) = args.get(pos + 1) {
            target = Some(t.clone());
            args.drain(pos..=pos + 1);
        }
    }

    // 環境変数をセットしてビルド先を指定
    if let Some(ref t) = target {
        env::set_var("CARGO_TARGET_DIR", format!("target/{}", t));
    }

    nih_plug_xtask::main()
}