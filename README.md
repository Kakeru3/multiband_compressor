# Multiband Compressor

## Building

.vst3ファイルの作り方

・mac&windowsで、じぶんのOSで使えるvstを生成する方法
> ```shell
> cargo xtask bundle multiband_compressor --release
> ```

・macでのwindows用.vs3コンパイル<br>
> xtask/src/main.rsを、
> ```
> use std::env;
> 
> fn main() -> nih_plug_xtask::Result<()> {
>     let mut args = env::args().collect::<Vec<_>>();
>     let mut target = None;
> 
>     if let Some(pos) = args.iter().position(|a| a == "--target") {
>         if let Some(t) = args.get(pos + 1) {
>             target = Some(t.clone());
>             args.drain(pos..=pos + 1);
>         }
>     }
> 
>     if let Some(ref t) = target {
>         env::set_var("CARGO_TARGET_DIR", format!("target/{}", t));
>     }
> 
>     nih_plug_xtask::main()
> }
> ```
> に変更し、
> ```shell
> cargo xtask bundle multiband_compressor --release --target x86_64-pc-windows-gnu
> ```
> で、target/x86_64-pc-windows-gnu/bundled/multiband_compressor.vst3/Contents/x86_64-win/
> に生成されます