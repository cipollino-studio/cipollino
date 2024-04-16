
cargo build --release --target x86_64-pc-windows-gnu

rm -Rf windows_x86_64_bundle
mkdir windows_x86_64_bundle

DIR=windows_x86_64_bundle/Cipollino

mkdir $DIR
cp target/x86_64-pc-windows-gnu/release/cipollino.exe $DIR
cp libs/bin/windows_x86/ffmpeg.exe $DIR/ffmpeg.exe